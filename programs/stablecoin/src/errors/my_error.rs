use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Health factor less than 1.0")]
    HealthFactorLessThanOne,
    #[msg("Health factor greater than 1.0")]
    HealthFactorGreaterThanOne,
    #[msg("Health factor greater than min health factor!")]
    HealthFactorGreaterMinHealthFactor,
}
