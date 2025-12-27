pub mod posts;

macro_rules! update_if_changed {
    (internal; $changed:expr, $query:expr, $prop:ident, $oldval:expr, $newval:expr, $newvalpush:expr) => {
        if $oldval != $newval {
            if $changed {
                $query.push(", ");
            }
            $query.push(format!(" {} = ", stringify!($prop)));
            $query.push_bind($newvalpush);

            $changed = true;
        };
    };

    ($changed:expr, $query:expr, $prop:ident, $old:expr, $new:expr) => {
        update_if_changed!(internal; $changed, $query, $prop, $old.$prop, *$new.$prop, $new.$prop);
    };

    // Skip dereferencing for certain types
    ($changed:expr, $query:expr, $prop:ident, $old:expr, $new:expr; skip_deref) => {
        update_if_changed!(internal; $changed, $query, $prop, $old.$prop, $new.$prop, $new.$prop);
    };
}

// Required for usage in this modules children
pub(self) use update_if_changed;
