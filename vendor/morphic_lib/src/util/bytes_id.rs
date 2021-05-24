macro_rules! bytes_id {
    (
        // Capturing attributes allows us to capture doc comments
        $(#[$annot_borrowed:meta])* $borrowed_vis:vis $borrowed:ident;
        $(#[$annot_owned:meta])* $owned_vis:vis $owned:ident;
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        $(#[$annot_borrowed])*
        $borrowed_vis struct $borrowed<'a>($borrowed_vis &'a [u8]);

        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        $(#[$annot_owned])*
        $owned_vis struct $owned($owned_vis ::std::vec::Vec<u8>);

        impl $owned {
            fn borrowed<'a>(&'a self) -> $borrowed<'a> {
                $borrowed(&self.0)
            }
        }

        impl<'a> ::std::convert::From<$borrowed<'a>> for $owned {
            fn from(borrowed: $borrowed<'a>) -> Self {
                $owned(<[u8]>::to_vec(&borrowed.0))
            }
        }
    }
}