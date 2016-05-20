macro_rules! items {
    ($($item:item)*) => ($($item)*)
}

macro_rules! trait_alias {
    ($vis:ident $name:ident = $($base:tt)+) => {
        items! {
            $vis trait $name: $($base)+ {}
            impl<T: $($base)+> $name for T {}
        }
    };
}
