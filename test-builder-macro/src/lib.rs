use proc_macro::{TokenStream, TokenTree, Delimiter};

#[proc_macro_attribute]
pub fn subtest(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut item = item.into_iter();

	if let Some(TokenTree::Ident(ident)) = item.next() {
		if ident.to_string() != "pub" {
			panic!()
		}
	} else {
		panic!()
	}

	if let Some(TokenTree::Ident(ident)) = item.next() {
		if ident.to_string() != "fn" {
			panic!()
		}
	} else {
		panic!()
	}

	if let Some(TokenTree::Group(group)) = item.next() {
		if group.delimiter() != Delimiter::Parenthesis &&
		if ident.to_string() != "fn" {
			panic!()
		}
	} else {
		panic!()
	}

	todo!()
}
