use crate::TransitGuard;

/// An error type that will be returned by the state machine if something goes wrong.
/// It fulfills the same purpose that the ordinary ```ignore SfsmError ``` does, but allows the
/// user to extend it with custom error types that are required by the fallible state machine.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExtendedSfsmError<T> {
    Internal,
    Custom(T)
}

/// Trait that must be implemented by all states that are used by the fallible state machine
/// Behaves similar to the normal ```ignore State ``` trait, but requires the user to specify
/// an Error type. If this error is returned, the state machine immediately transitions into the
/// error state.
pub trait TryState {
    type Error;
    fn try_entry(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn try_execute(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn try_exit(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

/// Trait that must be implemented by all states have a transition.
/// Behaves similar to the ```ignore TryTransition ``` trait.
pub trait TryTransition<DestinationState>: Into<DestinationState> + TryState {
    fn try_entry(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn try_execute(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn try_exit(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn guard(&self) -> TransitGuard;
}

/// This trait must be implemented by that state that is defined as the error state.
/// It allows the error state to consumed the error generated by another state for further
/// processing.
pub trait TryErrorState: TryState {
    fn consume_error(&mut self, err: Self::Error);
}