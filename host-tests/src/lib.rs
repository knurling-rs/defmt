pub fn add(left: u64, right: u64) -> u64 {
    defmt::info!("Adding {=u64} and {=u64}", left, right);
    let total = left + right;
    defmt::warn!("Got total {=u64}", total);
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
