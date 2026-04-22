#!/usr/bin/env python3
"""
send_nhs.py — отправляет .nhs пакет в NeroShizaDev-OS через serial-порт.

Протокол: NHS_SYNC -> NHS_READY -> [u32 LE: размер][<размер> байт NHS-данных] -> NHS_OK

Настройка QEMU (в run.bat или debug.bat замените -serial file:serial.log на):
    -serial tcp:127.0.0.1:4321,server,nowait

Тогда:
    python tools/send_nhs.py apps/installer/demo.nhs --tcp localhost:4321

Или через реальный COM-порт (UART-USB адаптер):
  python tools/send_nhs.py app.nhs --port COM3

В гостевой ОС набрать: install serial
"""

import sys
import struct
import socket
from pathlib import Path


SYNC_MAGIC = b"NHS_SYNC"
READY_MAGIC = b"NHS_READY"
OK_MAGIC = b"NHS_OK"
ERR_MAGIC = b"NHS_ERR"


def recv_until_token(sock: socket.socket, token: bytes, timeout: float = 10.0) -> bytes:
    sock.settimeout(timeout)
    received = bytearray()
    while token not in received and ERR_MAGIC not in received:
        chunk = sock.recv(64)
        if not chunk:
            raise ConnectionError("connection closed while waiting for handshake")
        received.extend(chunk)
    return bytes(received)


def recv_until_token_serial(ser, token: bytes, timeout: float = 10.0) -> bytes:
    ser.timeout = timeout
    received = bytearray()
    while token not in received and ERR_MAGIC not in received:
        chunk = ser.read(64)
        if not chunk:
            raise ConnectionError("serial timeout while waiting for handshake")
        received.extend(chunk)
    return bytes(received)


def send_via_tcp(host: str, port: int, nhs_path: str) -> None:
    data = Path(nhs_path).read_bytes()
    size = len(data)
    if size > 64 * 1024:
        print(f"[ERROR] File too large: {size} bytes (max 64 KB)", file=sys.stderr)
        sys.exit(1)

    print(f"[NHS] Connecting to {host}:{port}...", file=sys.stderr)
    with socket.create_connection((host, port), timeout=10) as sock:
        print("[NHS] Sending sync...", file=sys.stderr)
        sock.sendall(SYNC_MAGIC)
        reply = recv_until_token(sock, READY_MAGIC)
        if ERR_MAGIC in reply:
            raise ConnectionError("guest rejected sync")

        payload = struct.pack("<I", size) + data
        print(f"[NHS] Sending {nhs_path} ({size} bytes)...", file=sys.stderr)
        sock.sendall(payload)
        reply = recv_until_token(sock, OK_MAGIC)
        if ERR_MAGIC in reply:
            raise ConnectionError("guest rejected the payload")
        print("[NHS] Guest accepted payload. Watch QEMU installer screen.", file=sys.stderr)


def send_via_serial(port_path: str, baud: int, nhs_path: str) -> None:
    try:
        import serial  # type: ignore[import]
    except ImportError:
        print("[ERROR] pyserial not installed. Run: pip install pyserial", file=sys.stderr)
        sys.exit(1)

    data = Path(nhs_path).read_bytes()
    size = len(data)
    if size > 64 * 1024:
        print(f"[ERROR] File too large: {size} bytes (max 64 KB)", file=sys.stderr)
        sys.exit(1)

    print(f"[NHS] Opening {port_path} @ {baud} baud...", file=sys.stderr)
    with serial.Serial(port_path, baud, timeout=5) as ser:
        print("[NHS] Sending sync...", file=sys.stderr)
        ser.write(SYNC_MAGIC)
        ser.flush()
        reply = recv_until_token_serial(ser, READY_MAGIC)
        if ERR_MAGIC in reply:
            raise ConnectionError("guest rejected sync")

        payload = struct.pack("<I", size) + data
        print(f"[NHS] Sending {nhs_path} ({size} bytes)...", file=sys.stderr)
        ser.write(payload)
        ser.flush()
        reply = recv_until_token_serial(ser, OK_MAGIC)
        if ERR_MAGIC in reply:
            raise ConnectionError("guest rejected the payload")
        print("[NHS] Guest accepted payload.", file=sys.stderr)


def main() -> None:
    import argparse
    parser = argparse.ArgumentParser(
        description="Send .nhs package to NeroShizaDev-OS via serial",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument("nhs_file", help=".nhs file to send")
    parser.add_argument("--tcp", metavar="HOST:PORT",
                        help="QEMU TCP serial bridge, e.g. localhost:4321")
    parser.add_argument("--port", metavar="DEVICE",
                        help="Serial port, e.g. /dev/ttyUSB0 or COM3")
    parser.add_argument("--baud", type=int, default=115200,
                        help="Baud rate (default 115200)")
    args = parser.parse_args()

    nhs = args.nhs_file
    if not Path(nhs).exists():
        print(f"[ERROR] File not found: {nhs}", file=sys.stderr)
        sys.exit(1)

    if args.tcp:
        parts = args.tcp.rsplit(":", 1)
        if len(parts) != 2:
            print("[ERROR] --tcp expects HOST:PORT", file=sys.stderr)
            sys.exit(1)
        send_via_tcp(parts[0], int(parts[1]), nhs)
    elif args.port:
        send_via_serial(args.port, args.baud, nhs)
    else:
        print("[ERROR] Specify --tcp HOST:PORT or --port DEVICE", file=sys.stderr)
        print()
        print("QEMU TCP bridge setup (in run.bat replace -serial file:serial.log with):")
        print("  -serial tcp:127.0.0.1:4321,server,nowait")
        print()
        print("Then:")
        print("  python tools/send_nhs.py apps/installer/demo.nhs --tcp localhost:4321")
        sys.exit(1)


if __name__ == "__main__":
    main()
