mod instance;
mod types;

pub use ace_derive::AceInstance;
pub use instance::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_macro() {
        use crate as ace;

        #[derive(AceInstance)]
        #[constants(fippel = 5)]
        struct FippelInstance {
            #[db = "post.fippel"]
            _fippel: i64,
            #[db = "post.string"]
            _s: String,
        }

        assert_eq!(
            FippelInstance::FIELDS,
            &[("_fippel", "post.fippel"), ("_s", "post.string")]
        );
        assert_eq!(
            FippelInstance::constants().get("fippel").unwrap(),
            &AceValue::AceInt(5)
        );
    }
}
