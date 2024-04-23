use crate::db::traits::ExternalText;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Message {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 800))]
    pub message: String,
}

impl ExternalText for Message {
    fn cleaned(&self) -> Self {
        Self {
            email: self.email.to_owned(),
            message: self.clean(&self.message),
        }
    }
    fn clean(&self, value: &str) -> String {
        value
            .trim()
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_works() {
        let msg = Message {
            email: "me@abc.com".to_string(),
            message: "HI <b> Myname </b> <script> var p = 0; </script>".to_string(),
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_validation_message_fails() {
        let msg = Message {
            email: "me@abc.com".to_string(),
            // this string is longer than 800 characters
            message: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Integer malesuada, dui vel dignissim tempus, est purus gravida arcu, vitae \
            placerat ante purus in est. Nulla augue arcu, consequat a ultricies sit amet, \
            efficitur vel nisl. Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Sed in volutpat purus, sed congue quam. Pellentesque auctor ligula eget mi \
            laoreet semper. Donec volutpat lorem nec molestie aliquam. Class aptent taciti \
            sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. \
            Pellentesque in dolor at erat iaculis bibendum. Morbi gravida a ex nec tristique. \
            Donec commodo maximus tempor. Vivamus dapibus tincidunt est, sed sodales \
            quam dignissim ac. Mauris laoreet erat in venenatis scelerisque. Curabitur \
            vehicula lacinia luctus. Nam et tincidunt lacus. Aliquam lobortis vel arcu \
            sagittis volutpat."
                .to_string(),
        };
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_validation_email_works() {
        let msg = Message {
            email: "test@test.test".to_string(),
            message: "test".to_string(),
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_validation_email_fails() {
        let msg = Message {
            email: "@test.test".to_string(),
            message: "test".to_string(),
        };
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_validation_empty_message_fails() {
        let msg = Message {
            email: "me@abc.com".to_string(),
            message: "".to_string(),
        };
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_cleaned_works() {
        let msg = Message {
            email: "me@abc.com".to_string(),
            message: "HI <b> Myname </b> <script> var p = 0; </script>".to_string(),
        };
        let cleaned_msg = msg.cleaned();
        assert_eq!(cleaned_msg.email, msg.email);
        assert_eq!(cleaned_msg.message, "HI b Myname b script var p  0 script")
    }
}
