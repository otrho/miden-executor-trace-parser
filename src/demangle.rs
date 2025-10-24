// Typically the mangled names in these logs are weird, as the module name is not mangled, just the
// function path.  So we can search along until we hit a recognised mangle token and go from there.

pub(crate) fn demangle(mangled: &str) -> String {
    let mut demangled = String::with_capacity(mangled.len());

    enum ParseState {
        InPrefix,
        InPrefixWithUnderscore,
        AtMangleStart,
        AtPathStart,
        InPathElement,
        AtEnd,
    }

    let mut state = ParseState::InPrefix;
    let mut path_element = String::default();
    let mut path_element_count = 0_u32;

    for c in mangled.chars() {
        match state {
            ParseState::InPrefix => {
                if c == '_' {
                    state = ParseState::InPrefixWithUnderscore;
                } else {
                    demangled.push(c);
                }
            }

            ParseState::InPrefixWithUnderscore => {
                if c == 'Z' {
                    state = ParseState::AtMangleStart;
                } else {
                    demangled.push('_');
                    demangled.push(c);
                    state = ParseState::InPrefix;
                }
            }

            ParseState::AtMangleStart => {
                if c == 'N' {
                    state = ParseState::AtPathStart;
                } else {
                    todo!("Unexpected non-appearance of 'N' after '_Z'");
                }
            }

            ParseState::AtPathStart => {
                if let Some(n) = c.to_digit(10) {
                    path_element_count = path_element_count * 10 + n;
                } else if c == 'E' {
                    state = ParseState::AtEnd;
                } else {
                    if !path_element.is_empty() {
                        if !demangled.ends_with("::") {
                            demangled.push_str("::");
                        }
                        demangled.push_str(path_element.as_str());

                        path_element.clear();
                    }

                    path_element.push(c);
                    path_element_count -= 1;

                    if path_element_count > 0 {
                        state = ParseState::InPathElement;
                    }
                }
            }

            ParseState::InPathElement => {
                path_element.push(c);
                path_element_count -= 1;

                if path_element_count == 0 {
                    state = ParseState::AtPathStart;
                }
            }

            ParseState::AtEnd => {
                unreachable!("We shouldn't get characters after the end.");
            }
        }
    }

    demangled
}
