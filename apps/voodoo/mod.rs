/// Voodoo — диалог с нейро-оракулом.
///
///   человек ─► voodoo_nn (нейросеть: что сказано?)
///           ─► voodoo_engine (Байес + автоматы: что это значит?)
///           ─► chat (что ответить?)
///
///           ─► bridge (LLM на хосте через COM1: толкование и голос)
///           ─► interp (движок как интерпретатор выхода LLM: меньше бреда,
///                      больше жизни — версии на разных температурах)
///
/// Веса: weights/oracle.gguf, обучение: tools/voodoo_nn/train.py,
/// мост к llama.cpp: tools/voodoo_bridge/bridge.py.
pub mod bridge;
pub mod chat;
pub mod golden;
pub mod interp;
