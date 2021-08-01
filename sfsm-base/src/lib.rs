#![cfg_attr(not(test), no_std)]

/// Contains definitions for a state machine that contains error handling mechanisms
pub mod fallible;

/// Contains definitions used by a state machine without any error handling support
pub mod non_fallible;

pub mod __protected {
    // This trait will be implemented by the state machine itself.
    pub trait StateMachine {
        type InitialState;
        type Error;
        type StatesEnum;
        fn start(&mut self, state: Self::InitialState) -> Result<(), Self::Error>;
        fn step(&mut self) -> Result<(), Self::Error>;
        fn stop(self) -> Result<Self::StatesEnum, Self::Error>;
        fn peek_state(&self) -> &Self::StatesEnum;
    }
}

/// Enum used to indicate to the guard function if the transition should transit to the
/// next state or remain in the current one.
/// ```ignore
/// impl Transition<Bar> for Foo {
///     fn guard(&self) -> TransitGuard {
///         if self.foo == 0 {
///             TransitGuard::Transit
///         } else {
///             TransitGuard::Transit
///         }
///     }
/// }
/// ```
#[derive(PartialEq)]
pub enum TransitGuard {
    /// Remains in the current state
    Remain,
    // Transits into the next state
    Transit
}

/// Implements from<bool> trait for use of use.
/// This allows to transit by returning true. Which simplify the code since it allows to return the
/// TransitGuard from a simple comparison.
/// ```ignore
/// impl Transition<Bar> for Foo {
///     fn guard(&self) -> TransitGuard {
///         self.foo == 0 // Transits when self.foo == 0
///     }
/// }
/// ```
impl From<bool> for TransitGuard {
    fn from(transit: bool) -> Self {
        if transit {
            TransitGuard::Transit
        } else {
            TransitGuard::Remain
        }
    }
}

/// An implementation of this trait will be implemented for the state machine for every state.
/// This allows to test if the state machine is in the given state.
///
/// ```ignore
/// let is_in_state: bool = IsState::<State>::is_state(&sfsm);
/// ```
///
pub trait IsState<State>: __protected::StateMachine {
    fn is_state(&self) -> bool;
}

/// Error type that will be returned if an error during the message polling or pushing occurred.
/// It will indicate what the cause for the error was and return the original message in the push
/// case.
#[derive(Debug)]
#[non_exhaustive]
pub enum MessageError<T> {
    /// Will be returned if the state is not active. If it originated during a push, the rejected messaged will be returned with the error.
    StateIsNotActive(T),
}

/// Allows a state to declare that it can receive a message.
/// Note: for the state to actually be able to receive a message, the message has to be added with
/// the add_message! macro
pub trait ReceiveMessage<Message> {
    fn receive_message(&mut self, message: Message);
}

/// Allows a state to declare that it can return a message.
/// Note: for the state to actually be able to receive a message, the message has to be added with
/// the add_message! macro
pub trait ReturnMessage<Message> {
    fn return_message(&mut self) -> Option<Message>;
}

/// The PushMessage trait implementation will be generated by the add_message! macro and is used
/// to send messages into the state machine where they will then be forwarded to the correct
/// state.
///```ignore
/// use sfsm_base::PushMessage;
/// let some_message = 2u32;
/// PushMessage::<FooState, FooMessage>::push_message(&mut sfsm, some_message);
///```
/// This will call the receive_message function of 'FooState' if it implemented the ReceiveMessage
/// trait for message 'FooMessage' and it has been declared to do so with the add_message! macro.
pub trait PushMessage<State, Message>: __protected::StateMachine {
    fn push_message(&mut self, message: Message) -> Result<(), MessageError<Message>>;
}

/// The PollMessage trait implementation will be generated by the add_message! macro and is used
/// to return messages from states.
///```ignore
/// use sfsm_base::PollMessage;
/// let some_message = PollMessage::<FooState, FooMessage>::poll_message(&mut sfsm);
///```
/// This will call the return_message function of 'FooState' if it implemented the ReturnMessage
/// trait for message 'FooMessage' and it has been declared to do so with the add_message! macro.
pub trait PollMessage<State, Message>: __protected::StateMachine {
    fn poll_message(&mut self) -> Result<Option<Message>, MessageError<()>>;
}
