mod parsers;
mod generators;
mod types;

use quote::{quote};
use proc_macro::{TokenStream};
use types::Machine;
use crate::generators::StateMachineToTokens;
use crate::types::{IsState, MatchStateEntry};

/// Generates a state machine from a given state machine definition.
///
/// The state machine definition is expected too hold to the following pattern:
/// ```ignore
/// add_state_machine!(
///     StateMachineName,
///     InitialState,
///     [State1, State2, StateN, ...],
///     [StateN -> StateN, ...]
/// );
///```
/// So the following example:
/// ```ignore
/// add_state_machine!(
///         Elevator,
///         Move<Up>,
///         [Move<Up>, Move<Down>],
///         [
///             Move<Up> -> Move<Down>
///         ]
/// );
///```
/// will expand to this state machine.
///
///```ignore
/// enum ElevatorStates {
///     MoveUpState(Option<Move<Up>>),
///     MoveDownState(Option<Move<Down>>),
/// }
/// struct Elevator {
///     states: ElevatorStates,
///     do_entry: bool,
/// }
/// impl Elevator {
///     pub fn new(data: Move<Up>) -> Self {
///         Self {
///             states: ElevatorStates::MoveUpState(Some(data)),
///             do_entry: true,
///         }
///     }
///     pub fn step(&mut self) {
///         use ElevatorStates::*;
///         let ref mut e = self.states;
///         *e = match *e {
///             ElevatorStates::MoveUpState(ref mut state_option) => {
///                 let mut state = state_option.take().unwrap();
///                 if self.do_entry {
///                     State::entry(&mut state);
///                     Transition::<Move<Down>>::entry(&mut state);
///                     self.do_entry = false;
///                 }
///                 State::execute(&mut state);
///                 Transition::<Move<Down>>::execute(&mut state);
///                 if Transition::<Move<Down>>::guard(&state) {
///                     State::exit(&mut state);
///                     Transition::<Move<Down>>::exit(&mut state);
///                     let mut next_state: Move<Down> = state.into();
///                     self.do_entry = true;
///                     ElevatorStates::MoveDownState(Some(next_state))
///                 } else {
///                     ElevatorStates::MoveUpState(Some(state))
///                 }
///             }
///             ElevatorStates::MoveDownState(ref mut state_option) => {
///                 let mut state = state_option.take().unwrap();
///                 if self.do_entry {
///                     State::entry(&mut state);
///                     self.do_entry = false;
///                 }
///                 State::execute(&mut state);
///                 {
///                     ElevatorStates::MoveDownState(Some(state))
///                 }
///             }
///         }
///     }
///     pub fn peek_state(&self) -> &ElevatorStates {
///         return &self.states;
///     }
///     pub fn stop(mut self) -> ElevatorStates {
///         match self.states {
///             ElevatorStates::MoveUpState(ref mut state_option) => {
///                 let mut state = state_option.take().unwrap();
///                 State::exit(&mut state);
///                 Transition::<Move<Down>>::exit(&mut state);
///                 ElevatorStates::MoveUpState(Some(state))
///             }
///             ElevatorStates::MoveDownState(ref mut state_option) => {
///                 let mut state = state_option.take().unwrap();
///                 State::exit(&mut state);
///                 ElevatorStates::MoveDownState(Some(state))
///             }
///         }
///     }
/// }
///```
///
#[proc_macro]
pub fn add_state_machine(input: TokenStream) -> TokenStream {

    let definition = syn::parse_macro_input!(input as Machine);
    let sfsm_to_tokens = StateMachineToTokens::new(&definition);

    TokenStream::from(quote!{
        #sfsm_to_tokens
    })
}

/// Checks if the the state (as example returned by peek_state) is in the state to test.
/// ```ignore
/// let current_state = sfsm.peek_state();
/// assert!(is_state!(current_state, NameOfTheSfsm, DesiredState<AndType>));
/// ```
#[proc_macro]
pub fn is_state(input: TokenStream) -> TokenStream {

    let is_state: IsState = syn::parse_macro_input!(input as IsState);

    let state = is_state.state;
    let state_entry = is_state.state_entry;
    let enum_name = state_entry.enum_name;
    let state_entry = state_entry.state_entry;

    TokenStream::from(quote!{
         match #state {
             #enum_name::#state_entry(_) => {
                 true
             }
             _ => {
                 false
             }
         }
    })
}

/// Generate the enum entry of a state. Expects the name of the sfsm and the name (and type args)
/// of the state as well as the desired name of the variable to work with as arguments.
/// Can be used to generate match branches for example.
/// ```ignore
/// match exit {
///     match_state_entry!(NameOfTheSfsm, DesiredState<AndType>, var_name) => {
///         // Access "var_name" here.
///         // Var name will be Option<DesiredState<AndType>>
///     },
///     _ => {
///     }
/// }
/// ```
#[proc_macro]
pub fn match_state_entry(input: TokenStream) -> TokenStream {

    let match_state_entry: MatchStateEntry = syn::parse_macro_input!(input as MatchStateEntry);
    let state_entry = match_state_entry.state_entry;
    let enum_name = state_entry.enum_name;
    let state_entry = state_entry.state_entry;
    let var_name = match_state_entry.var_name;

    TokenStream::from(quote!{
        #enum_name::#state_entry(#var_name)
    })
}