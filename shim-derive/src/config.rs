use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprArray, ExprLit, FieldValue, Lit, Member, Result, Token};

#[derive(Default)]
pub struct Config {
    pub library: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let fields: Punctuated<FieldValue, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut config = Config::default();

        for field in fields {
            let Member::Named(name) = field.member else {
                return Err(Error::new_spanned(field.member, "field must be named"));
            };

            match name.to_string().as_str() {
                "library" => {
                    let Expr::Lit(ExprLit { lit: Lit::Str(library), .. }) = field.expr else {
                        return Err(Error::new_spanned(field.expr, "library must be a string literal"));
                    };

                    config.library = Some(library.value());
                }
                "include" => {
                    let Expr::Array(ExprArray { elems, .. }) = field.expr else {
                        return Err(Error::new_spanned(field.expr, "include must be an array"));
                    };

                    let include = elems.into_iter().map(|elem| {
                        let Expr::Lit(ExprLit { lit: Lit::Str(elem), .. }) = elem else {
                            return Err(Error::new_spanned(elem, "include must be a string literal"));
                        };

                        Ok(elem.value())
                    }).collect::<Result<Vec<_>>>()?;

                    config.include = Some(include);
                }
                "exclude" => {
                    let Expr::Array(ExprArray { elems, .. }) = field.expr else {
                        return Err(Error::new_spanned(field.expr, "exclude must be an array"));
                    };

                    let exclude = elems.into_iter().map(|elem| {
                        let Expr::Lit(ExprLit { lit: Lit::Str(elem), .. }) = elem else {
                            return Err(Error::new_spanned(elem, "exclude must be a string literal"));
                        };

                        Ok(elem.value())
                    }).collect::<Result<Vec<_>>>()?;

                    config.exclude = Some(exclude);
                }
                _ => {
                    return Err(Error::new_spanned(name, "unknown field"));
                }
            }
        }

        Ok(config)
    }
}
