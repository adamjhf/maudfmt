use anyhow::{Context, Result};
use crop::Rope;

mod collect;
mod format;
mod line_length;
mod print;
mod unparse;
mod vendor;

use vendor::ast;

pub use format::FormatOptions;

pub fn try_fmt_file(source: &str, options: &format::FormatOptions) -> Result<String> {
    let ast = syn::parse_file(source).context("Failed to parse source")?;
    let rope = Rope::from(source);
    let (mut rope, macros) = collect::collect_macros_from_file(&ast, rope, &options.macro_names);
    Ok(format::format_source(&mut rope, macros, options))
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use super::*;
    use crate::format::FormatOptions;
    use pretty_assertions::assert_eq;

    static DEFAULT_OPTIONS: LazyLock<FormatOptions> = LazyLock::new(FormatOptions::default);
    static SMALL_LINE_OPTIONS: LazyLock<FormatOptions> = LazyLock::new(|| FormatOptions {
        line_length: 40,
        ..Default::default()
    });

    macro_rules! test_default {
        ($title: ident, $content: literal, $expected: literal ) => {
            #[test]
            fn $title() {
                // check formatter works as expected
                assert_eq!(
                    try_fmt_file($content, &DEFAULT_OPTIONS).expect("should be able to parse"),
                    String::from($expected)
                );
                // check that `$expected` is a valid maud macro
                try_fmt_file($expected, &DEFAULT_OPTIONS)
                    .expect("expected should be parsable and valid maud");
            }
        };
    }

    macro_rules! test_small_line {
        ($title: ident, $content: literal, $expected: literal ) => {
            #[test]
            fn $title() {
                // check formatter works as expected
                assert_eq!(
                    try_fmt_file($content, &SMALL_LINE_OPTIONS).expect("should be able to parse"),
                    String::from($expected)
                );
                // check that `$expected` is a valid maud macro
                try_fmt_file($expected, &SMALL_LINE_OPTIONS)
                    .expect("expected should be parsable and valid maud");
            }
        };
    }

    test_default!(empty, "html!{ }", "html! {}");

    test_default!(
        empty_full_macro_declaration,
        "maud::html!{ }",
        "maud::html! {}"
    );

    test_default!(
        lit,
        r#"
        html!{ "Hello world!" }
        "#,
        r#"
        html! {
            "Hello world!"
        }
        "#
    );

    test_default!(
        escaping,
        r#"
        use maud::PreEscaped;
        html!{"<script>alert(\"XSS\")</script>" (PreEscaped("<script>alert(\"XSS\")</script>"))}
        "#,
        r#"
        use maud::PreEscaped;
        html! {
            "<script>alert(\"XSS\")</script>"
            (PreEscaped("<script>alert(\"XSS\")</script>"))
        }
        "#
    );

    test_default!(
        doctype,
        r#"
        use maud::DOCTYPE;
        html!{(DOCTYPE)}
        "#,
        r#"
        use maud::DOCTYPE;
        html! {
            (DOCTYPE)
        }
        "#
    );

    test_default!(
        elements_with_contents,
        r#"
        html! { h1 { "Poem" } p { strong { "Rock," } " you are a rock."}}
        "#,
        r#"
        html! {
            h1 { "Poem" }
            p {
                strong { "Rock," }
                " you are a rock."
            }
        }
        "#
    );

    test_default!(
        void_element,
        r#"
        html! {
          p { "Rock, you are a rock." br; "Gray, you are gray," br;
            "Like a rock, which you are." br; "Rock." } }
        "#,
        r#"
        html! {
            p {
                "Rock, you are a rock."
                br;
                "Gray, you are gray,"
                br;
                "Like a rock, which you are."
                br;
                "Rock."
            }
        }
        "#
    );

    test_default!(
        custom_elements_and_attributes,
        r#"
        html! {
          article data-index="12345"{h1 { "My blog"}tag-cloud {"pinkie pie pony cute"}}}
        "#,
        r#"
        html! {
            article data-index="12345" {
                h1 { "My blog" }
                tag-cloud { "pinkie pie pony cute" }
            }
        }
        "#
    );

    test_default!(
        non_empty_attributes,
        r#"
        html! { ul { li { a href="about:blank" { "Apple Bloom" } }
        li class="lower-middle" { "Sweetie Belle" }
        li dir="rtl" { "Scootaloo " small { "(also a chicken)" } } } }
        "#,
        r#"
        html! {
            ul {
                li {
                    a href="about:blank" { "Apple Bloom" }
                }
                li class="lower-middle" { "Sweetie Belle" }
                li dir="rtl" {
                    "Scootaloo "
                    small { "(also a chicken)" }
                }
            }
        }
        "#
    );

    test_default!(
        empty_attributes,
        r#"
        html! { form { input type="checkbox" name="cupcakes" checked;
        " " label for="cupcakes" { "Do you like cupcakes?" } } }
        "#,
        r#"
        html! {
            form {
                input type="checkbox" name="cupcakes" checked;
                " "
                label for="cupcakes" { "Do you like cupcakes?" }
            }
        }
        "#
    );

    test_default!(
        classes_and_ids,
        r#"
        html! { input#cannon .big.scary.bright-red type="button" value="Launch Party Cannon"; }
        "#,
        r#"
        html! {
            input #cannon.big.scary.bright-red type="button" value="Launch Party Cannon";
        }
        "#
    );

    test_default!(
        quoted_class_and_ids,
        r#"
        html!{div   #"quoted-id"   ."col-sm-2" { "Bootstrap column!" } }
        "#,
        r#"
        html! {
            div #"quoted-id"."col-sm-2" { "Bootstrap column!" }
        }
        "#
    );

    test_default!(
        implicit_div,
        r#"
        html! { #main { "Main content!" .tip { 
        "Storing food in a refrigerator can make it 20% cooler." } } }
        "#,
        r#"
        html! {
            #main {
                "Main content!"
                .tip { "Storing food in a refrigerator can make it 20% cooler." }
            }
        }
        "#
    );

    test_default!(
        splices,
        r#"
        html! { p { "Hi, " (best_pony) "!" }
            p{"I have "(numbers.len())" numbers, ""and the first one is "(numbers[0])}}
        "#,
        r#"
        html! {
            p { "Hi, " (best_pony) "!" }
            p { "I have " (numbers.len()) " numbers, " "and the first one is " (numbers[0]) }
        }
        "#
    );

    test_default!(
        splices_block,
        r#"
        html!{p{({
        let f: Foo = something_convertible_to_foo()?; f.time().format("%H%Mh") })}}
        "#,
        r#"
        html! {
            p {
                ({
                    let f: Foo = something_convertible_to_foo()?;
                    f.time().format("%H%Mh")
                })
            }
        }
        "#
    );

    test_default!(
        splice_in_attributes,
        r#"
        html!{p title=  (secret_message){"Nothing to see here, move along."}}
        "#,
        r#"
        html! {
            p title=(secret_message) { "Nothing to see here, move along." }
        }
        "#
    );

    test_default!(
        splice_concatenation,
        r#"
        html!{a href={(GITHUB)"/lambda-fairy/maud"}{"Fork me on GitHub"}}
        "#,
        r#"
        html! {
            a href={ (GITHUB) "/lambda-fairy/maud" } { "Fork me on GitHub" }
        }
        "#
    );

    test_default!(
        splice_classes_and_ids,
        r#"
        html!{aside #(name){p.{ "color-"(severity)}{"This is the worst! Possible! Thing!"}}}
        "#,
        r#"
        html! {
            aside #(name) {
                p.{ "color-" (severity) } { "This is the worst! Possible! Thing!" }
            }
        }
        "#
    );

    test_default!(
        toggle_empty_attributes,
        r#"
        html!{p contenteditable[allow_editing]{"Edit me, I " em{"dare"}" you."}}
        "#,
        r#"
        html! {
            p contenteditable[allow_editing] {
                "Edit me, I "
                em { "dare" }
                " you."
            }
        }
        "#
    );

    test_default!(
        toggle_classes,
        r#"
        html!{p.cute[cuteness>50]{"Squee!"}}
        "#,
        r#"
        html! {
            p.cute[cuteness > 50] { "Squee!" }
        }
        "#
    );

    test_default!(
        toggle_optional_attributes,
        r#"
        html!{p title=[Some("Good password")]{"Correct horse"}}
        "#,
        r#"
        html! {
            p title=[Some("Good password")] { "Correct horse" }
        }
        "#
    );

    test_default!(
        control_if,
        r#"
        html! { @if user == Princess::Luna {h1{"Super secret woona to-do list"}
        ul{li{"Nuke the Crystal Empire"}li{"Kick a puppy"}li{"Evil laugh"}}}}
        "#,
        r#"
        html! {
            @if user == Princess::Luna {
                h1 { "Super secret woona to-do list" }
                ul {
                    li { "Nuke the Crystal Empire" }
                    li { "Kick a puppy" }
                    li { "Evil laugh" }
                }
            }
        }
        "#
    );

    test_default!(
        control_if_else,
        r#"
        html! { @if user == Princess::Luna {h1{"Super secret woona to-do list"}
        ul{li{"Nuke the Crystal Empire"}li{"Kick a puppy"}li{"Evil laugh"}}}
        @else { p { "Nothing to see here; move along." } }}
        "#,
        r#"
        html! {
            @if user == Princess::Luna {
                h1 { "Super secret woona to-do list" }
                ul {
                    li { "Nuke the Crystal Empire" }
                    li { "Kick a puppy" }
                    li { "Evil laugh" }
                }
            } @else {
                p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        control_if_elseif_else,
        r#"
        html! { @if user == Princess::Luna {h1{"Super secret woona to-do list"}
        ul{li{"Nuke the Crystal Empire"}li{"Kick a puppy"}li{"Evil laugh"}}}
        @else if user==Princess::Celestia{p{"Sister, please stop reading my diary. It's rude."}}
        @else { p { "Nothing to see here; move along." } }}
        "#,
        r#"
        html! {
            @if user == Princess::Luna {
                h1 { "Super secret woona to-do list" }
                ul {
                    li { "Nuke the Crystal Empire" }
                    li { "Kick a puppy" }
                    li { "Evil laugh" }
                }
            } @else if user == Princess::Celestia {
                p { "Sister, please stop reading my diary. It's rude." }
            } @else {
                p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        if_let,
        r#"
        html! { p { "Hello, " @if let Some(name) = user { (name) } @else { "stranger" } "!"}}
        "#,
        r#"
        html! {
            p {
                "Hello, "
                @if let Some(name) = user { (name) } @else { "stranger" }
                "!"
            }
        }
        "#
    );

    test_default!(
        control_for,
        r#"
        html!{p{"My favorite ponies are:"}ol{@for name in &names{li{(name)}}}}
        "#,
        r#"
        html! {
            p { "My favorite ponies are:" }
            ol {
                @for name in &names {
                    li { (name) }
                }
            }
        }
        "#
    );

    test_default!(
        control_let,
        r#"
        html!{@for name in &names{@let first_letter=name.chars().next().unwrap();
        p{"The first letter of " b{(name)}" is " b{(first_letter)}"."}}}
        "#,
        r#"
        html! {
            @for name in &names {
                @let first_letter = name.chars().next().unwrap();
                p {
                    "The first letter of "
                    b { (name) }
                    " is "
                    b { (first_letter) }
                    "."
                }
            }
        }
        "#
    );

    test_default!(
        control_match,
        r#"
        html! { @match user { Princess::Luna => { h1 { "Super secret woona to-do list" } ul { li {
        "Nuke the Crystal Empire" } li { "Kick a puppy" } li { "Evil laugh" } } }, 
        Princess::Celestia => { p { "Sister, please stop reading my diary. It's rude." } }, _ => p
        { "Nothing to see here; move along." } } }
        "#,
        r#"
        html! {
            @match user {
                Princess::Luna => {
                    h1 { "Super secret woona to-do list" }
                    ul {
                        li { "Nuke the Crystal Empire" }
                        li { "Kick a puppy" }
                        li { "Evil laugh" }
                    }
                }
                Princess::Celestia => {
                    p { "Sister, please stop reading my diary. It's rude." }
                }
                _ => p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        control_match_with_guard,
        r#"
        html!{@match user{Princess::Luna if !is_asleep=>{h1{"Title"}
        h2{"Subtitle"}} _=>p{"Nothing to see here; move along."}}}
        "#,
        r#"
        html! {
            @match user {
                Princess::Luna if !is_asleep => {
                    h1 { "Title" }
                    h2 { "Subtitle" }
                }
                _ => p { "Nothing to see here; move along." }
            }
        }
        "#
    );

    test_default!(
        control_while,
        r#"
        html! { @while flag {p{"flag is true"}}}
        "#,
        r#"
        html! {
            @while flag {
                p { "flag is true" }
            }
        }
        "#
    );

    test_default!(
        control_while_let,
        r#"
        html! { @while let Some(value) = iter {p{(value)}}}
        "#,
        r#"
        html! {
            @while let Some(value) = iter {
                p { (value) }
            }
        }
        "#
    );

    test_default!(
        comment_inline,
        r##"
        use maud::DOCTYPE;
        html!{
        // <!DOCTYPE html>
        (DOCTYPE)
        }
        "##,
        r##"
        use maud::DOCTYPE;
        html! {
            // <!DOCTYPE html>
            (DOCTYPE)
        }
        "##
    );

    test_default!(
        comment_markup,
        r##"
        use maud::DOCTYPE;
        html!{
        (DOCTYPE)     // <!DOCTYPE html>
        }
        "##,
        r##"
        use maud::DOCTYPE;
        html! {
            (DOCTYPE)  // <!DOCTYPE html>
        }
        "##
    );

    test_default!(
        comment_empty_block,
        r#"
        html!{
            p {
                // lonely comment
            }
        }
        "#,
        r#"
        html! {
            p {
                // lonely comment
            }
        }
        "#
    );

    test_default!(
        keep_whitespace,
        r##"
        html!{
        "Hello"

        "World"
        }
        "##,
        r##"
        html! {
            "Hello"

            "World"
        }
        "##
    );

    test_default!(
        keep_single_whitespace,
        r##"
        html!{
        "Hello"



        "World"
        }
        "##,
        r##"
        html! {
            "Hello"

            "World"
        }
        "##
    );

    test_default!(
        force_expand_inline,
        r#"
        html! {
        h1 {
        // keep expanded
        "Poem"
        }
        }
        "#,
        r#"
        html! {
            h1 {
                // keep expanded
                "Poem"
            }
        }
        "#
    );

    test_default!(
        force_expand_attrs,
        r#"
        html! { 
        h1 { //
        "Poem"
        }
        }
        "#,
        r#"
        html! {
            h1 {  //
                "Poem"
            }
        }
        "#
    );

    test_default!(
        keep_comment_1,
        r#"
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    // meta
                    .first {}
                    .second {}
                }
            }
        }
        "#,
        r#"
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    // meta
                    .first {}
                    .second {}
                }
            }
        }
        "#
    );

    test_default!(
        comments_slashes_in_string,
        r#"
        html! {
            a href="http://example.org" { "This is not a comment" }
        }
        "#,
        r#"
        html! {
            a href="http://example.org" { "This is not a comment" }
        }
        "#
    );

    // NOTE: multiline string formatting is left to the users
    test_default!(
        whitespace_in_multi_line_strings_edge_case,
        r##"
        html! {
        p {
            (PreEscaped(r#"
            Multiline

            String
            "#))
        }
        }
        "##,
        r##"
        html! {
            p {
                ({
                    PreEscaped(
                        r#"
            Multiline

            String
            "#,
                    )
                })
            }
        }
        "##
    );

    // NOTE: multiline string formatting is left to the users
    test_default!(
        correct_multiline_string_indent_in_splices,
        r##"
        html! {
            (r#"
            Multiline
            String
            "#)
        }
        "##,
        r##"
        html! {
            ({
                r#"
            Multiline
            String
            "#
            })
        }
        "##
    );

    test_default!(
        line_length_long_splice,
        r##"
        html! {
            (super_long_splice.with_a_super_long_method().and_an_other_super_super_long_method_to_call_afer().unwarp())
        }
        "##,
        r##"
        html! {
            ({
                super_long_splice
                    .with_a_super_long_method()
                    .and_an_other_super_super_long_method_to_call_afer()
                    .unwarp()
            })
        }
        "##
    );

    test_small_line!(
        line_length_element_id,
        r##"
        html! {
        random-element#big-id-that-should-wrap {}
        }
        "##,
        r##"
        html! {
            random-element
                #big-id-that-should-wrap {}
        }
        "##
    );

    test_small_line!(
        line_length_class,
        r##"
        html! {
        random-element.class1.class2.class3 {}
        }
        "##,
        r##"
        html! {
            random-element
                .class1
                .class2
                .class3 {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_empty,
        r##"
        html! {
        random-element data-something-long {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something-long {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_empty_toggle,
        r##"
        html! {
        random-element data-something[true] {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something[true] {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_normal,
        r##"
        html! {
        random-element data-something="foo" {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something="foo" {}
        }
        "##
    );

    test_small_line!(
        line_length_attrs_optional,
        r##"
        html! {
        random-element data-something=[toggle] {}
        }
        "##,
        r##"
        html! {
            random-element
                data-something=[toggle] {}
        }
        "##
    );

    test_small_line!(
        line_length_element_body_no_expand,
        r##"
        html! {
            p { 
                "one line" 
            }
        }
        "##,
        r##"
        html! {
            p { "one line" }
        }
        "##
    );

    // NOTE: literal length is left to the user to deal with
    test_small_line!(
        line_length_element_body_expand_one_el,
        r##"
        html! {
            p { "one line very very long omg" }
        }
        "##,
        r##"
        html! {
            p {
                "one line very very long omg"
            }
        }
        "##
    );

    test_small_line!(
        line_length_element_body_no_expand_multi_el,
        r##"
        html! {
            p { 
                "one"
                "line"
            }
        }
        "##,
        r##"
        html! {
            p { "one" "line" }
        }
        "##
    );

    test_small_line!(
        line_length_element_body_expand_multi_el,
        r##"
        html! {
            p { "one very" "chunky line" }
        }
        "##,
        r##"
        html! {
            p {
                "one very"
                "chunky line"
            }
        }
        "##
    );

    test_small_line!(
        indented_multi_line_attribute_value,
        r#"
        html! {
            div test={ "This is a long multi-line attribute." "This is another line in the long attribute value." } {
                p { "hi" }
            }
        }
        "#,
        r#"
        html! {
            div
                test={
                    "This is a long multi-line attribute."
                    "This is another line in the long attribute value."
                } {
                p { "hi" }
            }
        }
        "#
    );

    test_default!(
        semi,
        r#"
        html! {
        ;
        }
        "#,
        r#"
        html! {
            ;
        }
        "#
    );

    test_default!(
        quoted_attributes,
        r#"
        html! {
            p "class"="bold" { "text" }
        }
        "#,
        r#"
        html! {
            p class="bold" { "text" }
        }
        "#
    );

    test_default!(
        quoted_attributes_special_chars,
        r#"
        html! {
            p "@click.window"="console.log('click')" "x-on:click"="test" ":class"="bold" { "click" }
        }
        "#,
        r#"
        html! {
            p "@click.window"="console.log('click')" x-on:click="test" ":class"="bold" { "click" }
        }
        "#
    );

    test_default!(
        keep_indents_in_comments_attrs,
        r#"
        html! {
        // p {
        //     "pls keep indent"
        // }
        p { }
        }
        "#,
        r#"
        html! {
            // p {
            //     "pls keep indent"
            // }
            p {}
        }
        "#
    );

    test_default!(
        keep_indents_in_comments_blocks,
        r#"
        html! {
        p { 
        // p {
        //     "pls keep indent"
        // }
        }
        }
        "#,
        r#"
        html! {
            p {
                // p {
                //     "pls keep indent"
                // }
            }
        }
        "#
    );

    test_default!(
        ensure_leading_space_in_comments,
        r#"
        html! {
        //please add leading space
        p { }
        }
        "#,
        r#"
        html! {
            // please add leading space
            p {}
        }
        "#
    );

    test_default!(
        rustfmt_skip,
        r#"
        #[rustfmt::skip]
        html! {
        p { }
        }
        "#,
        r#"
        #[rustfmt::skip]
        html! {
        p { }
        }
        "#
    );

    test_default!(
        rustfmt_skip_only_one,
        r#"
        html! {
        p { }
        }

        #[rustfmt::skip]
        html! {
        p { }
        }

        html! {
        p { }
        }
        "#,
        r#"
        html! {
            p {}
        }

        #[rustfmt::skip]
        html! {
        p { }
        }

        html! {
            p {}
        }
        "#
    );

    test_default!(
        rustfmt_skip_one_liner,
        r#"
        #[rustfmt::skip]
        html! {p{}}
        "#,
        r#"
        #[rustfmt::skip]
        html! {p{}}
        "#
    );
}
