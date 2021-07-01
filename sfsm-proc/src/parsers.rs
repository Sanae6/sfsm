use proc_macro2::{Ident, Span};
use proc_macro::{TokenStream};
use syn::{Result, AngleBracketedGenericArguments, Visibility, Attribute, Error};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::{Punctuated};
use syn::Token;
use convert_case::{Case, Casing};
use crate::types::{State, Transition, Machine, StateEntry, MatchStateEntry, StateMessage, Messages, Message, MessageDir};
use quote::ToTokens;
use syn::spanned::Spanned;

impl State {
    fn state_to_enum(name: &Ident, types: &Option<AngleBracketedGenericArguments>) -> Ident {
        let args_string = if let Some(args) = types {
            let mut args_string = args.into_token_stream().to_string();
            args_string = str::replace(args_string.as_str(), "'", "");
            args_string = str::replace(args_string.as_str(), "<", "");
            args_string = str::replace(args_string.as_str(), ">", "");
            args_string = str::replace(args_string.as_str(), "&", "");
            args_string = str::replace(args_string.as_str(), " ", "");
            args_string = str::replace(args_string.as_str(), ",", "");
            args_string = str::replace(args_string.as_str(), "]", "");
            args_string = str::replace(args_string.as_str(), "[", "");
            args_string.to_case(Case::Pascal)
        } else {
            "".to_string()
        };
        Ident::new(format!("{}{}State", name.to_string(), args_string).as_str(),
                   Span::call_site())
    }
}

/// Parses the name of a state and optionally a type.
/// For example Foo or Bar<T>
impl Parse for State {
    fn parse(input: ParseStream) -> Result<Self> {

        let name: Ident = input.parse()?;

        let generics = if input.peek(Token![<]) {
            input.parse::<AngleBracketedGenericArguments>().ok()
        } else {
            None
        };

        let enum_name = State::state_to_enum(&name, &generics);

        Ok(Self {
            name,
            transits: vec![],
            generics,
            enum_name,
        })
    }
}

/// Parses a transition that must be in the form of
/// Foo -> Bar or optionally with types like Foo<T> -> Bar<T>
impl Parse for Transition {
    fn parse(input: ParseStream) -> Result<Self> {
        let src: State = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        input.parse::<syn::Token![>]>()?;
        let dst: State = input.parse()?;

        Ok(Self {
            src,
            dst
        })
    }
}

impl Machine {
    pub fn enum_name(sfsm_name: &Ident) -> Ident {
        Ident::new(format!("{}States", sfsm_name.to_string()).as_str(),
                   Span::call_site())
    }
}

/// Parses the state machine in the form of
/// name, Foo, [Foo, Bar], [Foo -> Bar]
impl Parse for Machine {
    fn parse(input: ParseStream) -> Result<Self> {

        let attributes = input.call(Attribute::parse_outer).unwrap();

        let visibility: Option<Visibility> = input.parse().ok();

        let name: Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        let init: State = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        let state_group = input.parse::<proc_macro2::Group>()?;
        let state_group_ts: TokenStream = state_group.stream().into();
        let state_parser = Punctuated::<State, Token![,]>::parse_terminated;
        let punctuated_state_names = state_parser.parse(state_group_ts)?;
        let states_names: Vec<State> = punctuated_state_names.into_iter().collect();

        input.parse::<syn::Token![,]>()?;

        let transition_group = input.parse::<proc_macro2::Group>()?;
        let transition_group_ts: TokenStream = transition_group.stream().into();
        let transition_parser =
            Punctuated::<Transition, Token![,]>::parse_terminated;
        let punctuated_transitions = transition_parser.parse(transition_group_ts)?;
        let transitions: Vec<Transition> = punctuated_transitions.into_iter().collect();

        let states: Vec<State> = states_names.into_iter().map(|state| {

            let transitions: Vec<State> = (&transitions).into_iter().filter(|trans| {
                return trans.src.enum_name == state.enum_name;
            }).map(|trans| (*trans).dst.clone()).collect();

            State {
                name: state.name,
                transits: transitions,
                generics: state.generics,
                enum_name: state.enum_name,
            }

        }).collect();

        let enum_name = Machine::enum_name(&name);

        Ok(Self {
            attributes,
            visibility,
            name,
            init,
            states,
            enum_name,
        })
    }
}

impl Parse for StateEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let sfsm_name: Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let state: State = input.parse()?;

        Ok(Self {
            enum_name: Machine::enum_name(&sfsm_name),
            state_entry: state.enum_name,
        })
    }
}

impl Parse for MatchStateEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let state_entry: StateEntry = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let var_name: Ident = input.parse()?;
        Ok(Self {
            state_entry,
            var_name
        })
    }
}
impl Parse for Message {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        // Only parse the generic argument if the bracket is opened and no - follows.
        // If we only checked for the < the arrow <- would trigger the parsing.
        let generics = if input.peek(Token![<])
                    && !input.peek2(Token![-]) {
            input.parse::<AngleBracketedGenericArguments>().ok()
        } else {
            None
        };

        Ok(Self {
            name,
            generics
        })
    }
}

impl Parse for StateMessage {
    fn parse(input: ParseStream) -> Result<Self> {
        let message: Message = input.parse()?;

        let message_dir: MessageDir = if input.peek(syn::Token![->]) {
            input.parse::<syn::Token![->]>()?;
            MessageDir::Push(message)
        } else if input.peek(syn::Token![<-]) {
            input.parse::<syn::Token![<-]>()?;
            MessageDir::Poll(message)
        } else {
            return Err(Error::new(input.span(), format!("A direction must be specified with either '->' or '<-' but got '{}' instead", input)))
        };

        let state: State = input.parse()?;
        Ok(Self {
            message: message_dir,
            state
        })
    }
}


/// Parses the message definitions in the form of
/// name, [M1 -> Foo, M2 <- Bar]
impl Parse for Messages {
    fn parse(input: ParseStream) -> Result<Self> {

        let name: Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        let state_message_group = input.parse::<proc_macro2::Group>()?;
        let state_message_group_ts: TokenStream = state_message_group.stream().into();
        let state_message_parser = Punctuated::<StateMessage, Token![,]>::parse_terminated;
        let punctuated_state_names = state_message_parser.parse(state_message_group_ts)?;
        let messages: Vec<StateMessage> = punctuated_state_names.into_iter().collect();

        let enum_name = Machine::enum_name(&name);

        Ok(Self {
            name,
            enum_name,
            messages
        })
    }
}