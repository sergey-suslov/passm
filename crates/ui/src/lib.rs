mod components;
mod event_loop;
pub mod ui;
pub mod widgets;
pub use event_loop::EventLoop;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
