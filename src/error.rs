use crate::State;

pub type RResult<T, S> = Result<T, RunError<<S as State>::Error>>;

#[derive(Debug, thiserror::Error)]
pub enum InvalidFileError {
    #[error("statement calls procedure that doesn't exist in file")]
    CantFindProcedureForCall,
    #[error("found expression or comparison block when statement was expected")]
    FoundExpressionWhenStmtExpected(smodel::Id),
    #[error("procedure call doesn't provide all arguments needed")]
    MissingArgForProcedureParameter,
    #[error("procedure call has extra arguments not expected by definition")]
    ExtraArgWithoutProcedureParameter,
    #[error("block reference doesn't point to an existing block")]
    NoBlockFoundForReference(smodel::error::BlockReferenceInvalid),
    #[error("next block reference points to no block in file")]
    ReachedUnknownBlock(smodel::Id),
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("without a body no data can change, so a condition-based loop will never stop")]
    ConditionLoopWithoutBodyNeverStops,
    #[error("a single-threadded wait-until block called with false will never continue")]
    WaitUntilFalseNeverTerminatesInSingleThreadded,
    #[error("an infinite loop without a body can never halt")]
    InfiniteLoopWithoutBodyNeverStops,
}
#[derive(Debug, thiserror::Error)]
pub enum UnsupportedError {
    #[error("blocks related to cloning are not supported")]
    CloneBlocksNotSupported(smodel::Id),
    #[error("sending of events is not unsupported")]
    EventSendingNotSupported,
    #[error("uses of unsupported sensing block")]
    UnsupportedSensingBlock,
    #[error("uses of unsupported looks block")]
    UnsupportedLooksBlock,
    #[error("uses of unsupported motion block")]
    UnsupportedMotionBlock,
}

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("saved instruction asks to remove stack frame, but no frames are stored")]
    PopOnEmptyArgumentFramesStack,
    /// Should never occur, this indicates a error in the interpreter
    #[error("unexpected kind of stack frame found for this context")]
    FatalWrongStackFrameForContext,
    /// Should never occur, stack is checked before popping
    #[error("program stack doesn't contain another instruction")]
    PopOnEmptyProgramStack,
}

#[derive(Debug, thiserror::Error)]
pub enum LimitError {
    #[error("a string operation produced a text that is longer than allowed")]
    StringExceededLengthLimit(usize),
    #[error("executed more statements than allowed")]
    ExceededAllowedStmtCount,
    #[error("stack size exceeds limit")]
    MaxStackSizeExceeded,
}

#[derive(Debug, thiserror::Error)]
pub enum RunError<SErr> {
    #[error("state: {0}")]
    State(SErr),
    #[error("terminated by stop block")]
    TerminatedByControlStop,
    #[error("limit: {0}")]
    Limit(#[from] LimitError),
    #[error("internal: {0}")]
    Internal(#[from] InternalError),
    #[error("damaged-file: {0}")]
    File(#[from] InvalidFileError),
    #[error("user: {0}")]
    User(#[from] UserError),
    #[error("unsupported: {0}")]
    Unsupported(#[from] UnsupportedError),
}
