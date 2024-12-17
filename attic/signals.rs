// use tokio::signal;

// #[cfg(windows)]
// {
//     pub enum SignalHandlers{
//         CtrlC(signal::windows::CtrlC),
//         CtrlClose(signal::windows::CtrlClose),
//         CtrlBreak(signal::windows::CtrlBreak),
//         CtrlShutdown(signal::windows::CtrlShutdown),
//         CtrlLogoff(signal::windows::CtrlLogoff),

//     }
// // }

// struct Signals{
//     handlers: Vec<SignalHandlers>,
//     signals: Option<Vec<SignalHandlers>>
// }

// impl Signals {
//     fn get(&mut self) -> Vec<SignalHandlers>{
//         self.signals.take().unwrap()
//     }
// }

// pub fn get_signals() -> Signals{

//     let signal_handlers: Vec<SignalHandlers> = Vec::new();

//     #[cfg(windows)]
//     {
//         signal_handlers.push(SignalHandlers::CtrlC(signal::windows::ctrl_c().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlBreak(signal::windows::ctrl_break().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlClose(signal::windows::ctrl_close().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlLogoff(signal::windows::ctrl_logoff().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlShutdown(signal::windows::ctrl_shutdown().unwrap()));
//     }

//     #[cfg(unix)]
//     {

//     }

//     let signals = signal_handlers.iter().map(|s| s.recv()).collect::<Vec>();

//     let signals_struct = Signals{
//         handlers: signal_handlers,
//         signals: Some(signals)
//     };

//     signals_struct
// }

// pub async fn wait_for_signals(){

//     let signal_handlers: Vec<SignalHandlers> = Vec::new();

//     #[cfg(windows)]
//     {
//         signal_handlers.push(SignalHandlers::CtrlC(signal::windows::ctrl_c().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlBreak(signal::windows::ctrl_break().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlClose(signal::windows::ctrl_close().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlLogoff(signal::windows::ctrl_logoff().unwrap()));
//         signal_handlers.push(SignalHandlers::CtrlShutdown(signal::windows::ctrl_shutdown().unwrap()));
//     }

//     #[cfg(unix)]
//     {

//     }

//     let signals = signal_handlers.iter().map(|s| s.recv()).collect::<Vec>();

//     let signals_struct = Signals{
//         handlers: signal_handlers,
//         signals: Some(signals)
//     };

//     signals_struct
// }