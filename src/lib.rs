use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1, take_while1},
    character::complete::{alphanumeric1, digit1, multispace0, multispace1},
    combinator::{map, map_res, opt},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

#[derive(Debug)]
pub enum Definition<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
    Alias(Field<'a>),
}

impl<'a> Definition<'a> {
    pub fn name(&self) -> &str {
        match self {
            Definition::Struct(r#struct) => r#struct.name,
            Definition::Enum(r#enum) => r#enum.name,
            Definition::Alias(alias) => alias.name,
        }
    }
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: &'a str,
    pub items: Vec<FieldOrSelect<'a>>,
}

#[derive(Debug)]
pub enum FieldOrSelect<'a> {
    Field(Field<'a>),
    Select(Select<'a>),
}

#[derive(Debug)]
pub struct Field<'a> {
    pub r#type: &'a str,
    pub name: &'a str,
    pub range: Option<Range<'a>>,
    pub optional: bool,
    pub default: Option<&'a str>,
}

#[derive(Debug)]
pub enum Range<'a> {
    MinMax((usize, usize)),
    Exact(usize),
    Variable,
    Prose(&'a str),
}

#[derive(Debug)]
pub struct Select<'a> {
    pub over: &'a str,
    pub cases: Vec<Case<'a>>,
}

#[derive(Debug)]
pub struct Case<'a> {
    pub left: &'a str,
    pub right: CaseBody<'a>,
}

#[derive(Debug)]
pub enum CaseBody<'a> {
    /// Example: `struct{};`
    Empty,
    /// Example: `All;`
    ReferenceToType(&'a str),
    /// Example: `opaque value;`
    /// Note: Empty is also allowed. Fallthrough?
    Fields(Vec<Field<'a>>),
}

/// Note: We could enforce that there is only one `EnumItemMax` and simplify this.
#[derive(Debug)]
pub struct Enum<'a> {
    pub name: &'a str,
    pub items: Vec<EnumItem<'a>>,
}

/// Note: We could enforce that there is only one
#[derive(Debug)]
pub enum EnumItem<'a> {
    Value((&'a str, &'a str)),
    Max(&'a str),
}

// -------------------------------------------------------------------------------------------------

fn is_type_character(c: char) -> bool {
    c.is_alphanumeric()
}

fn is_field_name_character(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// -------------------------------------------------------------------------------------------------

fn r#struct(input: &str) -> IResult<&str, Struct> {
    let mut parser = tuple((
        tag("struct"),
        multispace1,
        tag("{"),
        many0(delimited(multispace0, field_or_select, multispace0)),
        tag("}"),
        multispace0,
        take_while1(|c| c != ';'),
        tag(";"),
    ));

    let (remaining, (_, _, _, items, _, _, name, _)) = parser(input)?;

    Ok((remaining, Struct { name, items }))
}

fn field_or_select(input: &str) -> IResult<&str, FieldOrSelect> {
    alt((
        map(field, FieldOrSelect::Field),
        map(select, FieldOrSelect::Select),
    ))(input)
}

fn field(input: &str) -> IResult<&str, Field> {
    let mut parser = tuple((
        r#type,
        multispace1,
        take_while1(is_field_name_character),
        opt(alt((
            delimited(
                tag("["),
                alt((
                    map(map_res(digit1, str::parse), Range::Exact),
                    map(take_until1("]"), Range::Prose),
                )),
                tag("]"),
            ),
            delimited(tag("<"), alt((fixed, variable)), tag(">")),
        ))),
        opt(map(
            tuple((multispace0, tag("="), multispace0, take_until1(";"))),
            |(_, _, _, default)| default,
        )),
        tag(";"),
    ));

    let (remaining, ((r#type, optional), _, name, range, default, _)) = parser(input)?;

    Ok((
        remaining,
        Field {
            r#type,
            name,
            range,
            optional,
            default,
        },
    ))
}

fn r#type(input: &str) -> IResult<&str, (&str, bool)> {
    alt((
        map(
            preceded(
                tag("optional"),
                delimited(tag("<"), take_while1(is_type_character), tag(">")),
            ),
            |v| (v, true),
        ),
        map(take_while1(is_type_character), |v| (v, false)),
    ))(input)
}

fn fixed(input: &str) -> IResult<&str, Range> {
    let mut parser = map(
        separated_pair(
            map_res(digit1, str::parse),
            tag(".."),
            map_res(digit1, str::parse),
        ),
        |(start, end): (usize, usize)| (start, end),
    );

    let (remaining, fixed) = parser(input)?;

    Ok((remaining, Range::MinMax(fixed)))
}

fn variable(input: &str) -> IResult<&str, Range> {
    let parser = tag("V");

    let (remaining, _) = parser(input)?;

    Ok((remaining, Range::Variable))
}

fn select(input: &str) -> IResult<&str, Select> {
    let mut parser = tuple((
        tag("select"),
        multispace1,
        delimited(
            tag("("),
            delimited(multispace0, take_while1(|c| c != ')'), multispace0),
            tag(")"),
        ),
        multispace0,
        tag("{"),
        many0(delimited(multispace0, case, multispace0)),
        tag("}"),
        multispace0,
        tag(";"),
    ));

    let (remaining, (_, _, over, _, _, cases, _, _, _)) = parser(input)?;

    Ok((remaining, Select { over, cases }))
}

fn case(input: &str) -> IResult<&str, Case> {
    let mut parser = tuple((
        tag("case"),
        multispace1,
        take_while1(|s| s != ':'),
        tag(":"),
        alt((
            map(
                delimited(multispace0, tag("struct{};"), multispace0),
                |_| CaseBody::Empty,
            ),
            map(
                tuple((multispace0, alphanumeric1, tag(";"), multispace0)),
                |(_, r#type, _, _)| CaseBody::ReferenceToType(r#type),
            ),
            map(
                many0(delimited(multispace0, field, multispace0)),
                CaseBody::Fields,
            ),
        )),
    ));

    let (remaining, (_, _, left, _, right)) = parser(input)?;

    Ok((remaining, Case { left, right }))
}

// -------------------------------------------------------------------------------------------------

fn r#enum(input: &str) -> IResult<&str, Enum> {
    let mut parser = tuple((
        tag("enum"),
        multispace1,
        tag("{"),
        separated_list1(tag(","), delimited(multispace0, enum_item, multispace0)),
        tag("}"),
        multispace0,
        take_while1(|c| c != ';'),
        tag(";"),
    ));

    let (remaining, (_, _, _, items, _, _, name, _)) = parser(input)?;

    Ok((remaining, Enum { name, items }))
}

fn enum_item(input: &str) -> IResult<&str, EnumItem> {
    let mut parser = alt((
        map(
            tuple((
                take_while1(is_field_name_character),
                delimited(tag("("), digit1, tag(")")),
            )),
            |(name, value)| EnumItem::Value((name, value)),
        ),
        map(delimited(tag("("), digit1, tag(")")), |max| {
            EnumItem::Max(max)
        }),
    ));

    let (remaining, item) = parser(input)?;

    Ok((remaining, item))
}

// -------------------------------------------------------------------------------------------------

pub fn parse(input: &str) -> Result<Vec<Definition>, String> {
    let (rem, out) = full(input).map_err(|_| "Parsing failed".to_string())?;

    if !rem.is_empty() {
        return Err(format!("Trailing data detected:\n{}", rem));
    }

    Ok(out)
}

// -------------------------------------------------------------------------------------------------

fn full(input: &str) -> IResult<&str, Vec<Definition>> {
    delimited(
        multispace0,
        separated_list0(multispace0, definition),
        multispace0,
    )(input)
}

fn definition(input: &str) -> IResult<&str, Definition> {
    alt((
        map(r#struct, Definition::Struct),
        map(r#enum, Definition::Enum),
        map(field, Definition::Alias),
    ))(input)
}

// -------------------------------------------------------------------------------------------------

pub mod dependencies {
    use crate::{Case, CaseBody, Definition, FieldOrSelect};

    pub trait Dependencies {
        fn dependencies(&self) -> Vec<&str>;
    }

    impl<'a> Dependencies for Definition<'a> {
        fn dependencies(&self) -> Vec<&str> {
            let mut deps = match self {
                Definition::Struct(r#struct) => r#struct
                    .items
                    .iter()
                    .map(Dependencies::dependencies)
                    .flatten()
                    .collect(),
                Definition::Enum(_) => {
                    vec![]
                }
                Definition::Alias(alias) => {
                    vec![alias.r#type]
                }
            };

            deps.sort();
            deps.dedup();

            deps
        }
    }

    impl<'a> Dependencies for FieldOrSelect<'a> {
        fn dependencies(&self) -> Vec<&str> {
            match self {
                FieldOrSelect::Field(field) => {
                    vec![field.r#type]
                }
                FieldOrSelect::Select(select) => select
                    .cases
                    .iter()
                    .map(Dependencies::dependencies)
                    .flatten()
                    .collect(),
            }
        }
    }

    impl<'a> Dependencies for Case<'a> {
        fn dependencies(&self) -> Vec<&str> {
            match &self.right {
                CaseBody::Empty => {
                    vec![]
                }
                CaseBody::Fields(fields) => fields.iter().map(|field| field.r#type).collect(),
                CaseBody::ReferenceToType(r#type) => {
                    vec![r#type]
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn that_enum_can_be_parsed() {
        let tests = [
            "enum {
    reserved(0),
    mls10(1),
    (255)
} ProtocolVersion;",
            "enum {
    reserved(0),
    application(1),
    proposal(2),
    commit(3),
    (255)
} ContentType;",
            "enum {
    reserved(0),
    member(1),
    external(2),
    new_member_proposal(3),
    new_member_commit(4),
    (255)
} SenderType;",
        ];

        for test in tests {
            let (remainder, object) = r#enum(test).unwrap();
            assert!(remainder.is_empty());

            println!("{}\n{:#?}\n\n", test, object);
        }
    }

    #[test]
    fn that_enum_item_can_be_parsed() {
        let tests = [
            "reserved(0)",
            "mls10(1)",
            "application(1)",
            "proposal(2)",
            "commit(3)",
            "(255)",
            "member(1)",
            "external(2)",
            "new_member_proposal(3)",
            "new_member_commit(4)",
            "(255)",
        ];

        for test in tests {
            let (remainder, object) = enum_item(test).unwrap();
            assert!(remainder.is_empty());

            println!("{}\n{:?}\n\n", test, object);
        }
    }

    #[test]
    fn that_struct_can_be_parsed() {
        let tests = [
            "struct {
    uint8 present;
    select (present) {
        case 0: struct{};
        case 1: T value;
    };
} optional<T>;",
            "struct {
    uint32 fixed<0..255>;
    opaque variable<V>;
} StructWithVectors;",
            "struct {
    opaque cert_data<V>;
} Certificate;",
            "struct {
    CredentialType credential_type;
    select (Credential.credential_type) {
        case basic:
            opaque identity<V>;

        case x509:
            Certificate chain<V>;
    };
} Credential;",
        ];

        for test in tests {
            let (remainder, object) = r#struct(test).unwrap();
            assert!(remainder.is_empty());

            println!("{}\n{:#?}\n\n", test, object);
        }
    }

    #[test]
    fn that_field_or_select_be_parsed() {
        let tests = [
            "select (present) { case 0: struct{}; case 1: T value; };",
            "uint8 present;",
        ];

        for test in tests {
            let (remainder, object) = field_or_select(test).unwrap();
            assert!(remainder.is_empty());

            println!("{}\n{:?}\n\n", test, object);
        }
    }

    #[test]
    fn that_select_can_be_parsed() {
        let tests = ["select (present) { case 0: struct{}; case 1: T value; };"];

        for test in tests {
            let (remainder, object) = select(test).unwrap();
            assert!(remainder.is_empty());

            println!("{}\n{:?}\n\n", test, object);
        }
    }

    #[test]
    fn that_case_can_be_parsed() {
        let tests = [
            "case 0: struct{};",
            "case 1: T value;",
            "case basic:
            opaque identity<V>;

",
            "case x509:
            Certificate chain<V>;",
            "case foo:",
            "case bar: opaque group_id<V>;
            uint32 leaf_index;",
            "case add:                      Add;",
        ];

        for test in tests {
            let (remainder, object) = case(test).unwrap();
            println!("<{}>", remainder);
            assert!(remainder.is_empty());

            println!("{}\n{:?}\n\n", test, object);
        }
    }

    #[test]
    fn that_field_can_be_parsed() {
        let tests = [
            "uint8 present;",
            "uint32 fixed<0..255>;",
            "opaque variable<V>;",
            "opaque cert_data<V>;",
            "CredentialType credential_type;",
            "optional<LeafNode> leaf_node;",
            "ProtocolVersion version = mls10;",
            "opaque padding[length_of_padding];",
        ];

        for test in tests {
            let (remainder, object) = field(test).unwrap();
            assert!(remainder.is_empty());

            println!("{}\n{:?}\n\n", test, object);
        }
    }
}
