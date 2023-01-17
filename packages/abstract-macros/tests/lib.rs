#[cfg(test)]
mod tests {
    use super::*;
    use abstract_event::abstract_response;
    use cosmwasm_std::Response;
    use speculoos::prelude::*;

    #[test]
    fn test_abstract_response() {
        const CONTRACT_NAME: &str = "abstract:contract";
        const ACTION: &str = "instantiate";
        let actual: Response = abstract_response!(CONTRACT_NAME, ACTION);
        let expected = Response::new().add_event(
            cosmwasm_std::Event::new("abstract")
                .add_attributes(vec![("contract", CONTRACT_NAME), ("action", ACTION)]),
        );
        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn test_addition_to_response() {
        const CONTRACT_NAME: &str = "abstract:contract";
        const ACTION: &str = "instantiate";
        let new_attributes = vec![("who dat who dat", "IGGY")];
        let actual: Response =
            abstract_response!(CONTRACT_NAME, ACTION).add_attributes(new_attributes.clone());
        let expected = Response::new()
            .add_event(cosmwasm_std::Event::new("abstract").add_attributes(vec![
                ("contract", "abstract:contract"),
                ("action", "instantiate"),
            ]))
            .add_attributes(new_attributes);
        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn test_with_quoted_attributes() {
        let actual: Response =
            abstract_response!("abstract:contract", "instantiate", [("custom", "abstract")]);
        let expected =
            Response::new().add_event(cosmwasm_std::Event::new("abstract").add_attributes(vec![
                ("contract", "abstract:contract"),
                ("action", "instantiate"),
                ("custom", "abstract"),
            ]));
        assert_that!(actual).is_equal_to(expected);
    }
}
