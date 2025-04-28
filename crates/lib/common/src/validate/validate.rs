use bigdecimal::{BigDecimal, Signed};
use validator::ValidationError;

/// 验证 `BigDecimal` 是否为正数且大于0
pub fn validate_positive_decimal(data: &BigDecimal) -> Result<(), ValidationError> {
    if data.is_positive() && data > &BigDecimal::from(0) {
        Ok(())
    } else {
        let mut error = ValidationError::new("positive_decimal");
        error.message = Some("输入参数必须大于0".into());
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validate_positive_decimal_test() {
        let data = BigDecimal::from(100);
        let res = validate_positive_decimal(&data);
        assert!(res.is_ok());

        let zero = BigDecimal::from(0);
        let res = validate_positive_decimal(&zero);
        assert!(res.is_err());

        let negative = BigDecimal::from(-1);
        let res = validate_positive_decimal(&negative);
        assert!(res.is_err());
    }
}
