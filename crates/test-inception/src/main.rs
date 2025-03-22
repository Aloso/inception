inception::rules! {
    match struct as (/* $meta* */ $vis struct $name:ident $generics:generic_params? $struct_rest);

    match vis as (pub) | (pub($vis_inner)) | ();
    match vis_inner as (crate) | (self) | (super) | (in $path);
    match path as ();

    match generic_params as (< ${generic_param , .. ,}? >);
    match generic_param /*..*/ as
        | (/* $meta* */ $ident $colon_type_bounds?)
        | (/* $meta* */ $lifetime $colon_lifetime_bounds?)
        | (/* $meta* */ const $ident $: $ty);
    match colon_type_bounds /*..*/ as (: $bounds:{bound + ..}?);
    match colon_lifetime_bounds /*..*/ as (: $bounds:{lifetime + ..}?);
    match bound as ($lifetime) | ($ty);

    match where_clause as (where $where_bounds*);
    match where_bounds /*..*/ as
        | (/* $meta* */ $ty $colon_type_bounds)
        | (/* $meta* */ $lifetime $colon_lifetime_bounds);

    match struct_rest /*..*/ as
        | ($where_clause? $body:struct_body)
        | ($body:tuple_struct_body? $where_clause? ;);

    match struct_body as ({ $fields:{struct_field , .. ,}? });
    match tuple_struct_body as (( $fields:{tuple_struct_field , .. ,}? ));
    match struct_field as (/* $meta* */ $vis $ident $: $ty);
    match tuple_struct_field as ($meta* $vis $ty);

    pub macro Foo1($s: struct) {
        /// This struct was parsed and expanded again by inception!
        ${s.meta}
        ${s.vis} struct ${s.name} ${s.generics}
        $if s.is_regular_struct {
            ${s.where_clause} ${s.struct_rest.body}
        } $else {
            ${s.struct_rest.body} ${s.where_clause};
        }
    }
}

#[inception::attr(Foo1)]
struct Bar {}

fn main() {
    let _bar = Bar {};
}

/*
inception::rules! {
    pub macro Foo($s: struct) {
        /// This struct was parsed and expanded again by inception!
        ${s.meta}
        ${s.vis} struct ${s.name} ${s.generics}
        $if s.is_regular_struct {
            ${s.where_clause} ${s.body}
        } $else {
            ${s.body} ${s.where_clause};
        }
    }
}
*/

/*
inception::rules! {
    match foo as
        | (.: $literal+)
        | (,; $literal+);

    macro Foo2(. $foo+ $ident $+ $lifetime+ : ${tt , .. ,}? $tt2:{tt , .. ,}?) {
        const _: () = {
            fn $ident<
                $for lt in lifetime { $lt, }
                T: $for lt in lifetime { $lt + }
            >() -> & ${first(lifetime)} str {
                ${last(foo.literal)} ${tt}
            }
        };
    }
}

Foo2! {
    . ,; "test" "test" ,; "lol" bar + 'a 'b 'c :
}

fn main() {}
*/
