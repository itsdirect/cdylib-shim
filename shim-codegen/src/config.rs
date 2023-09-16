use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprLit, FieldValue, Lit, Member, Path, Result, Token};

#[derive(Default)]
pub struct Config {
    pub library: Option<String>,
    pub load: Option<Path>,
    pub unload: Option<Path>,
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
                "load" => {
                    let Expr::Path(path) = field.expr else {
                        return Err(Error::new_spanned(field.expr, "load must be a path"));
                    };

                    config.load = Some(path.path);
                }
                "unload" => {
                    let Expr::Path(path) = field.expr else {
                        return Err(Error::new_spanned(field.expr, "unload must be a path"));
                    };

                    config.unload = Some(path.path);
                }
                _ => {
                    return Err(Error::new_spanned(name, "unknown field"));
                }
            }
        }

        Ok(config)
    }
}
