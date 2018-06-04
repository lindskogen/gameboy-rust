#![feature(nll)]

mod dmg;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let d = super::dmg::Core::start();
        assert_eq!(2 + 2, 4);
    }
}
