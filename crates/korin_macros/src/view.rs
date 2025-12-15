use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};

pub fn implmentation(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);

    match parse_node(&mut input.into_iter().peekable()) {
        Ok(node) => generate_node(&node).into(),
        Err(err) => err.into(),
    }
}

#[derive(Debug)]
enum Node {
    Element(Element),
    Fragment(Vec<Self>),
    Expr(TokenStream2),
}

#[derive(Debug)]
struct Element {
    name: String,
    props: Vec<Prop>,
    children: Vec<Node>,
}

#[derive(Debug)]
struct Prop {
    name: String,
    value: TokenStream2,
}

type TokenIter = std::iter::Peekable<proc_macro2::token_stream::IntoIter>;

fn parse_node(tokens: &mut TokenIter) -> Result<Node, TokenStream2> {
    match tokens.peek() {
        Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
            tokens.next(); // consume '<'

            // Check for fragment <>
            if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '>') {
                tokens.next(); // consume '>'
                return parse_fragment(tokens);
            }

            parse_element(tokens)
        }
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
            let TokenTree::Group(g) = tokens.next().expect("group already peeked") else {
                unreachable!("already matched as group")
            };

            Ok(Node::Expr(g.stream()))
        }
        Some(TokenTree::Literal(_)) => {
            let TokenTree::Literal(lit) = tokens.next().expect("literal already peeked") else {
                unreachable!("already matched as literal")
            };
            Ok(Node::Expr(quote! { #lit }))
        }
        Some(other) => {
            let s = other.to_string();
            Err(quote! {
                compile_error!(concat!("expected '<', '{...}', or string literal, got: ", #s))
            })
        }
        None => Err(quote! {
            compile_error!("unexpected end of input")
        }),
    }
}

fn parse_fragment(tokens: &mut TokenIter) -> Result<Node, TokenStream2> {
    let mut children = Vec::new();

    loop {
        // Check for closing </>
        if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '<') {
            let mut lookahead = tokens.clone();
            lookahead.next();
            if matches!(lookahead.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '/') {
                lookahead.next();
                if matches!(lookahead.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '>') {
                    // Consume </>
                    tokens.next();
                    tokens.next();
                    tokens.next();
                    break;
                }
            }
        }

        if tokens.peek().is_none() {
            return Err(quote! { compile_error!("unclosed fragment") });
        }

        children.push(parse_node(tokens)?);
    }

    Ok(Node::Fragment(children))
}

fn parse_element(tokens: &mut TokenIter) -> Result<Node, TokenStream2> {
    // Get element name
    let name = match tokens.next() {
        Some(TokenTree::Ident(ident)) => ident.to_string(),
        _ => return Err(quote! { compile_error!("expected element name") }),
    };

    let mut props = Vec::new();
    let mut children = Vec::new();

    // Parse props and detect self-closing or children
    loop {
        match tokens.peek() {
            // Self-closing />
            Some(TokenTree::Punct(p)) if p.as_char() == '/' => {
                tokens.next();
                expect_punct(tokens, '>')?;
                break;
            }
            // Children >
            Some(TokenTree::Punct(p)) if p.as_char() == '>' => {
                tokens.next();
                children = parse_children(tokens, &name)?;
                break;
            }
            // Event: on:name={handler}
            Some(TokenTree::Ident(ident)) if *ident == "on" => {
                tokens.next();
                expect_punct(tokens, ':')?;
                let event_name = expect_ident(tokens)?;
                expect_punct(tokens, '=')?;
                let value = expect_braced(tokens)?;
                props.push(Prop {
                    name: format!("on_{event_name}"),
                    value,
                });
            }
            // Shorthand prop: {name}
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let TokenTree::Group(g) = tokens.next().expect("group already peeked") else {
                    unreachable!("already matched as group")
                };
                let inner = g.stream().to_string();
                props.push(Prop {
                    name: inner.clone(),
                    value: g.stream(),
                });
            }
            // Regular prop: name={value}
            Some(TokenTree::Ident(_)) => {
                let prop_name = expect_ident(tokens)?;
                expect_punct(tokens, '=')?;
                let value = expect_braced(tokens)?;
                props.push(Prop {
                    name: prop_name,
                    value,
                });
            }
            Some(other) => {
                let s = other.to_string();
                return Err(quote! { compile_error!(concat!("unexpected token: ", #s)) });
            }
            None => return Err(quote! { compile_error!("unexpected end of input in element") }),
        }
    }

    Ok(Node::Element(Element {
        name,
        props,
        children,
    }))
}

fn parse_children(tokens: &mut TokenIter, parent_name: &str) -> Result<Vec<Node>, TokenStream2> {
    let mut children = Vec::new();

    loop {
        // Check for closing tag </Name>
        if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '<') {
            let mut lookahead = tokens.clone();
            lookahead.next();
            if matches!(lookahead.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '/') {
                lookahead.next();
                // Consume </Name>
                if let Some(TokenTree::Ident(ident)) = lookahead.peek()
                    && *ident == parent_name
                {
                    tokens.next(); // <
                    tokens.next(); // /
                    tokens.next(); // Name
                    expect_punct(tokens, '>')?;
                    break;
                }
            }
        }

        if tokens.peek().is_none() {
            return Err(quote! { compile_error!("unclosed element") });
        }

        children.push(parse_node(tokens)?);
    }

    Ok(children)
}

fn expect_punct(tokens: &mut TokenIter, expected: char) -> Result<(), TokenStream2> {
    match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == expected => Ok(()),
        _ => Err(quote! { compile_error!(concat!("expected '", #expected, "'")) }),
    }
}

fn expect_ident(tokens: &mut TokenIter) -> Result<String, TokenStream2> {
    match tokens.next() {
        Some(TokenTree::Ident(ident)) => Ok(ident.to_string()),
        _ => Err(quote! { compile_error!("expected identifier") }),
    }
}

fn expect_braced(tokens: &mut TokenIter) -> Result<TokenStream2, TokenStream2> {
    match tokens.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => Ok(g.stream()),
        _ => Err(quote! { compile_error!("expected {...}") }),
    }
}

fn generate_node(node: &Node) -> TokenStream2 {
    match node {
        Node::Element(elem) => generate_element(elem),
        Node::Fragment(children) => generate_fragment(children),
        Node::Expr(expr) => quote! { korin_view::IntoView::into_view((#expr)) },
    }
}

fn generate_element(elem: &Element) -> TokenStream2 {
    let name = format_ident!("{}", elem.name);
    let props_name = format_ident!("{}Props", elem.name);

    let prop_setters: Vec<_> = elem
        .props
        .iter()
        .map(|p| {
            let name = format_ident!("{}", p.name);
            let value = &p.value;

            if p.name.starts_with("on_") {
                quote! { .#name(korin_event::IntoHandler::into_handler(#value)) }
            } else if p.name == "style" {
                quote! { .#name(korin_view::IntoAnyStyle::<korin_runtime::RuntimeContext>::into_style(#value)) }
            }
            else {
                quote! { .#name(#value) }
            }
        })
        .collect();

    let children_code = if elem.children.is_empty() {
        quote! {}
    } else {
        let child_exprs: Vec<_> = elem.children.iter().map(generate_node).collect();
        quote! { .children(vec![#(#child_exprs),*]) }
    };

    quote! {
        {
            let result = #name(#props_name::builder()
                #(#prop_setters)*
                #children_code
                .build()
            );

            korin_view::IntoView::into_view(result)
        }
    }
}

fn generate_fragment(children: &[Node]) -> TokenStream2 {
    let child_exprs: Vec<_> = children.iter().map(generate_node).collect();
    quote! {
        vec![#(#child_exprs),*]
    }
}
