use crate::{Error, Result};
use get_if_addrs::{get_if_addrs, Interface};
use roxmltree::{Document, Node};
use std::net::{IpAddr, SocketAddrV4};

pub trait HttpResponseExt: Sized {
    fn err_if_not_200(self) -> Result<Self>;
}
impl HttpResponseExt for http::Response<isahc::Body> {
    fn err_if_not_200(self) -> Result<Self> {
        if self.status() != 200 {
            Err(Error::HttpErrorCode(self.status()))
        } else {
            Ok(self)
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! find_in_xml {
    ( $node:expr => $( $($var:ident)? $(?$var_opt:ident)? ),+ ) => { {
        let node = $node;
        $(
            $(let mut $var = None;)?
            $(let mut $var_opt = None;)?
        )*
        for child in node.children().filter(Node::is_element) {
            match child.tag_name().name() {
                $(
                    $(stringify!($var) => $var = Some(child),)?
                    $(stringify!($var_opt) => $var_opt = Some(child),)?
                )*
                _ => (),
            }
        }

        $(
            $(let $var = $var.ok_or_else(|| crate::Error::XMLMissingElement(
                    node.tag_name().name().to_string(),
                    stringify!($var).to_string(),
                ))?;)?
        )*

        ($(
            $($var)?
            $($var_opt)?
        ),*)
    } }
}

pub fn parse_node_text<T, E>(node: Node<'_, '_>) -> Result<T>
where
    T: std::str::FromStr<Err = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    node.text()
        .unwrap_or_default()
        .parse()
        .map_err(Error::invalid_response)
}

pub fn find_root<'a, 'input: 'a>(
    document: &'input Document<'_>,
    element: &str,
    docname: &str,
) -> Result<Node<'a, 'input>> {
    document
        .descendants()
        .filter(Node::is_element)
        .find(|n| n.tag_name().name().eq_ignore_ascii_case(element))
        .ok_or_else(|| Error::XMLMissingElement(docname.to_string(), element.to_string()))
}

pub fn get_local_addr() -> SocketAddrV4 {
    let addr = get_if_addrs()
        .unwrap()
        .iter()
        .map(Interface::ip)
        .filter_map(|addr| match addr {
            IpAddr::V4(addr) => Some(addr),
            IpAddr::V6(_) => None,
        })
        .find(|x| x.is_private())
        .expect("no local ipv4 interface open");
    SocketAddrV4::new(addr, 0)
}