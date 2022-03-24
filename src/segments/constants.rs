// Segment Rules
pub const ALL_RULE: &str = "ALL";
pub const ANY_RULE: &str = "ANY";
pub const NONE_RULE: &str = "NONE";

const RULE_TYPES :[&str; 3] = [ALL_RULE, ANY_RULE, NONE_RULE];

// Segment Condition Operators
pub const EQUAL: &str = "EQUAL";
pub const GREATER_THAN: &str = "GREATER_THAN";
pub const LESS_THAN: &str = "LESS_THAN";
pub const LESS_THAN_INCLUSIVE: &str = "LESS_THAN_INCLUSIVE";
pub const CONTAINS: &str = "CONTAINS";
pub const GREATER_THAN_INCLUSIVE: &str = "GREATER_THAN_INCLUSIVE";
pub const NOT_CONTAINS: &str = "NOT_CONTAINS";
pub const NOT_EQUAL: &str = "NOT_EQUAL";
pub const REGEX: &str = "REGEX";
pub const PERCENTAGE_SPLIT: &str = "PERCENTAGE_SPLIT";

const CONDITION_OPERATORS: [&str; 10] = [
    EQUAL,
    GREATER_THAN,
    LESS_THAN,
    LESS_THAN_INCLUSIVE,
    CONTAINS,
    GREATER_THAN_INCLUSIVE,
    NOT_CONTAINS,
    NOT_EQUAL,
    REGEX,
    PERCENTAGE_SPLIT,
];
