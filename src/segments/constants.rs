// Segment Rules
const ALL_RULE: &str = "ALL";
const ANY_RULE: &str = "ANY";
const NONE_RULE: &str = "NONE";

const RULE_TYPES: Vec<&str> = [ALL_RULE, ANY_RULE, NONE_RULE];

// Segment Condition Operators
const EQUAL: &str = "EQUAL";
const GREATER_THAN: &str = "GREATER_THAN";
const LESS_THAN: &str = "LESS_THAN";
const LESS_THAN_INCLUSIVE: &str = "LESS_THAN_INCLUSIVE";
const CONTAINS: &str = "CONTAINS";
const GREATER_THAN_INCLUSIVE: &str = "GREATER_THAN_INCLUSIVE";
const NOT_CONTAINS: &str = "NOT_CONTAINS";
const NOT_EQUAL: &str = "NOT_EQUAL";
const REGEX: &str = "REGEX";
const PERCENTAGE_SPLIT: &str = "PERCENTAGE_SPLIT";

const CONDITION_OPERATORS: Vec<&str> = [
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
