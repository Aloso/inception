macro_rules! better_macro_rules {
    ($($t:tt)*) => {};
}

mod better_macro {
    pub(crate) use better_macro_rules as rules;
    pub(crate) use better_macro_rules as derive;
}

better_macro::rules! {
    use rust_parser::v1;

    // imports all of the following:

    match struct as ($meta* $vis struct $name:ident $generics:generic_params? $struct_rest);

    match generic_params as (< ${generic_param , .. ,}? >);
    match generic_param.. as
        | ($meta* $ident $colon_type_bounds?)
        | ($meta* $lifetime $colon_lifetime_bounds?)
        | ($meta* const $ident $: $ty);
    match colon_type_bounds.. as (: $bounds:{bound + ..}?);
    match colon_lifetime_bounds.. as (: $bounds:{lifetime + ..}?);
    match bound as ($lifetime) | ($ty);

    match where_clause as (where $where_bounds*);
    match where_bounds.. as
        | ($meta* $ty $colon_type_bounds)
        | ($meta* $lifetime $colon_lifetime_bounds);

    fn generic_params.get_names() {
        $for param in generic_params {
            $match param {
                Type { ident, .. } => { $ident , }
                Lifetime { lifetime, .. } => { $lifetime , }
                Const { ident, .. } => { $ident , }
            }
        }
    }

    match struct_rest.. as
        | ($where_clause? $body:struct_body)
        | ($body:tuple_struct_body? $where_clause? ;);

    match struct_body as ({ $fields:{struct_field , .. ,}? });
    match tuple_struct_body as (( $fields:{tuple_struct_field , .. ,}? ));
    match struct_field as ($meta* $vis $ident $: $ty);
    match tuple_struct_field as ($meta* $vis $ty);

    #[derive]
    pub macro TypeString($s: struct) {
        impl <${s.generics}> TypeString for ${s.name} <${s.generics.get_names()}>
        ${s.where_clause}
        {
            fn type_string() -> &'static str {
                $match s.body {
                    Struct { fields, .. } => { ${fields.concat(${s.name})} }
                    Tuple { fields, .. } => { ${fields.concat(${s.name})} }
                }
            }
        }
    }

    macro struct_body.concat($ident: ident) {
        concat!(
            stringify!($ident),
            " { ",
            $for field, i in struct_body.field {
                stringify!(${field.ident}),
                $if !i.is_last() {
                    ", ",
                }
            },
            " }"
        )
    }

    macro tuple_struct_body.concat($ident: ident) {
        concat!(
            stringify!($ident),
            "(",
            $for field, i in struct_body.field {
                stringify!(${field.ident}),
                $if !i.is_last() {
                    ", ",
                }
            },
            ")"
        )
    }
}

trait TypeString {
    fn type_string() -> &'static str;
}

#[derive(Debug)]
// #[better_macro::derive(TypeString)]
struct Bar<T> {
    a: i32,
    b: T,
}

impl<T> TypeString for Bar<T> {
    fn type_string() -> &'static str {
        concat!(stringify!(Bar), " { ", stringify!(a), ", ", stringify!(b), " }")
    }
}

fn main() {
    assert_eq!("Bar { a, b }", Bar::<()>::type_string());
}
