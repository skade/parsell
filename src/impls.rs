//! Provide implementations of parser traits.

use super::{Parser, ParseResult};
use super::{HasOutput, StatefulInfer, Stateful, CommittedInfer, Committed, UncommittedInfer, Uncommitted, Boxable};
use super::{Function, VariantFunction, Consumer, Factory, PeekableIterator};
use super::{Upcast, Downcast, ToStatic};
use super::ParseResult::{Done, Continue};

use self::OrElseState::{Lhs, Rhs};
use self::AndThenState::{InLhs, InBetween, InRhs};

use std::borrow::Cow;
use std::borrow::Cow::{Borrowed, Owned};
use std::str::Chars;
use std::fmt::{Formatter, Debug};
use std;

// ----------- N-argument functions ---------------

#[derive(Copy, Clone, Debug)]
pub struct Function2<F>(F);

impl<F> Function2<F> {
    pub fn new(f: F) -> Self {
        Function2(f)
    }
}

// NOTE(eddyb): a generic over U where F: Fn(T) -> U doesn't allow HRTB in both T and U.
// See https://github.com/rust-lang/rust/issues/30867 for more details.
impl<F, S1, S2, O> Function<(S1, S2)> for Function2<F> where F: Fn(S1, S2) -> O
{
    type Output = F::Output;
    fn apply(&self, args: (S1, S2)) -> F::Output {
        (self.0)(args.0, args.1)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Function3<F>(F);

impl<F> Function3<F> {
    pub fn new(f: F) -> Self {
        Function3(f)
    }
}

// NOTE(eddyb): a generic over U where F: Fn(T) -> U doesn't allow HRTB in both T and U.
// See https://github.com/rust-lang/rust/issues/30867 for more details.
impl<F, S1, S2, S3, O> Function<((S1, S2), S3)> for Function3<F> where F: Fn(S1, S2, S3) -> O
{
    type Output = F::Output;
    fn apply(&self, args: ((S1, S2), S3)) -> F::Output {
        (self.0)((args.0).0, (args.0).1, args.1)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Function4<F>(F);

impl<F> Function4<F> {
    pub fn new(f: F) -> Self {
        Function4(f)
    }
}

// NOTE(eddyb): a generic over U where F: Fn(T) -> U doesn't allow HRTB in both T and U.
// See https://github.com/rust-lang/rust/issues/30867 for more details.
impl<F, S1, S2, S3, S4, O> Function<(((S1, S2), S3), S4)> for Function4<F>
    where F: Fn(S1, S2, S3, S4) -> O
{
    type Output = F::Output;
    fn apply(&self, args: (((S1, S2), S3), S4)) -> F::Output {
        (self.0)(((args.0).0).0, ((args.0).0).1, (args.0).1, args.1)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Function5<F>(F);

impl<F> Function5<F> {
    pub fn new(f: F) -> Self {
        Function5(f)
    }
}

// NOTE(eddyb): a generic over U where F: Fn(T) -> U doesn't allow HRTB in both T and U.
// See https://github.com/rust-lang/rust/issues/30867 for more details.
impl<F, S1, S2, S3, S4, S5, O> Function<((((S1, S2), S3), S4), S5)> for Function5<F>
    where F: Fn(S1, S2, S3, S4, S5) -> O
{
    type Output = F::Output;
    fn apply(&self, args: ((((S1, S2), S3), S4), S5)) -> F::Output {
        (self.0)((((args.0).0).0).0,
                 (((args.0).0).0).1,
                 ((args.0).0).1,
                 (args.0).1,
                 args.1)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Function6<F>(F);

impl<F> Function6<F> {
    pub fn new(f: F) -> Self {
        Function6(f)
    }
}

// NOTE(eddyb): a generic over U where F: Fn(T) -> U doesn't allow HRTB in both T and U.
// See https://github.com/rust-lang/rust/issues/30867 for more details.
impl<F, S1, S2, S3, S4, S5, S6, O> Function<(((((S1, S2), S3), S4), S5), S6)> for Function6<F>
    where F: Fn(S1, S2, S3, S4, S5, S6) -> O
{
    type Output = F::Output;
    fn apply(&self, args: (((((S1, S2), S3), S4), S5), S6)) -> F::Output {
        (self.0)(((((args.0).0).0).0).0,
                 ((((args.0).0).0).0).1,
                 (((args.0).0).0).1,
                 ((args.0).0).1,
                 (args.0).1,
                 args.1)
    }
}

// ----------- Deal with errors ---------------

#[derive(Copy, Clone, Debug)]
pub struct Try<F>(F);
impl<F, S, E> Function<Result<S, E>> for Try<F> where F: Function<S>
{
    type Output = Result<F::Output,E>;
    fn apply(&self, args: Result<S, E>) -> Result<F::Output, E> {
        Ok(self.0.apply(try!(args)))
    }
}
impl<F, T, E> VariantFunction<Result<T, E>> for Try<F> where F: VariantFunction<T>
{
    type Input = Result<F::Input,E>;
    fn apply(&self, args: Result<F::Input, E>) -> Result<T, E> {
        Ok(self.0.apply(try!(args)))
    }
}
impl<F> Try<F> {
    pub fn new(f: F) -> Try<F> {
        Try(f)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TryDiscard;
impl<S, E> Function<Result<S, E>> for TryDiscard {
    type Output = Result<(),E>;
    fn apply(&self, arg: Result<S, E>) -> Result<(), E> {
        try!(arg); Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TryZip;
impl<S, T, E> Function<(Result<S, E>, T)> for TryZip {
    type Output = Result<(S,T),E>;
    fn apply(&self, args: (Result<S, E>, T)) -> Result<(S, T), E> {
        Ok((try!(args.0), args.1))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ZipTry;
impl<S, T, E> Function<(S, Result<T, E>)> for ZipTry {
    type Output = Result<(S,T),E>;
    fn apply(&self, args: (S, Result<T, E>)) -> Result<(S, T), E> {
        Ok((args.0, try!(args.1)))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TryZipTry;
impl<S, T, E> Function<(Result<S, E>, Result<T, E>)> for TryZipTry {
    type Output = Result<(S,T),E>;
    fn apply(&self, args: (Result<S, E>, Result<T, E>)) -> Result<(S, T), E> {
        Ok((try!(args.0), try!(args.1)))
    }
}
impl<S, T, E> VariantFunction<Result<(S, T), E>> for TryZipTry {
    type Input = (Result<S, E>, Result<T, E>);
    fn apply(&self, args: (Result<S, E>, Result<T, E>)) -> Result<(S, T), E> {
        Ok((try!(args.0), try!(args.1)))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TryOpt;
impl<T, E> Function<Option<Result<T, E>>> for TryOpt {
    type Output = Result<Option<T>,E>;
    fn apply(&self, arg: Option<Result<T, E>>) -> Result<Option<T>,E> {
        match arg {
            Some(Ok(res)) => Ok(Some(res)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }
}
impl<T, E> VariantFunction<Result<Option<T>, E>> for TryOpt {
    type Input = Option<Result<T, E>>;
    fn apply(&self, arg: Option<Result<T, E>>) -> Result<Option<T>,E> {
        match arg {
            Some(Ok(res)) => Ok(Some(res)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }
}

// ----------- Deal with options ---------------

#[derive(Copy, Clone, Debug)]
pub struct MkSome;
impl<T> Function<T> for MkSome
{
    type Output = Option<T>;
    fn apply(&self, arg: T) -> Option<T> {
        Some(arg)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IsSome<F>(F);
impl<F, S, T> Function<S> for IsSome<F>
    where F: Function<S, Output = Option<T>>
{
    type Output = bool;
    fn apply(&self, arg: S) -> bool {
        self.0.apply(arg).is_some()
    }
}
impl<F> IsSome<F> {
    pub fn new(f: F) -> IsSome<F> {
        IsSome(f)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Unwrap<F>(F);
impl<F, S, T> Function<S> for Unwrap<F>
    where F: Function<S, Output = Option<T>>
{
    type Output = T;
    fn apply(&self, arg: S) -> T {
        self.0.apply(arg).unwrap()
    }
}
impl<F> Unwrap<F> {
    pub fn new(f: F) -> Unwrap<F> {
        Unwrap(f)
    }
}

// ----------- Deal with dereferencing ---------------

#[derive(Copy, Clone, Debug)]
pub struct Dereference<F>(F);
impl<F, S, T> Function<S> for Dereference<F>
    where F: for<'a> Function<&'a S, Output = T>
{
    type Output = T;
    fn apply(&self, arg: S) -> T {
        self.0.apply(&arg)
    }
}
impl<F> Dereference<F> {
    pub fn new(f: F) -> Dereference<F> {
        Dereference(f)
    }
}


// ----------- Deal with pairs ---------------

#[derive(Copy, Clone, Debug)]
pub struct First;
impl<S, T> Function<(S, T)> for First
{
    type Output = S;
    fn apply(&self, arg: (S, T)) -> S {
        arg.0
    }
}
impl<T> VariantFunction<T> for First
{
    type Input = (T, ());
    fn apply(&self, arg: (T, ())) -> T {
        arg.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Second;
impl<S, T> Function<(S, T)> for Second
{
    type Output = T;
    fn apply(&self, arg: (S, T)) -> T {
        arg.1
    }
}
impl<T> VariantFunction<T> for Second
{
    type Input = ((), T);
    fn apply(&self, arg: ((), T)) -> T {
        arg.1
    }
}


// ----------- Map ---------------

pub struct Map<P, F>(P, F);

// A work around for functions implmenting copy but not clone
// https://github.com/rust-lang/rust/issues/28229
impl<P, F> Copy for Map<P, F>
    where P: Copy,
          F: Copy
{}
impl<P, F> Clone for Map<P, F>
    where P: Clone,
          F: Copy
{
    fn clone(&self) -> Self {
        Map(self.0.clone(), self.1)
    }
}

// A work around for named functions not implmenting Debug
// https://github.com/rust-lang/rust/issues/31522
impl<P, F> Debug for Map<P, F>
    where P: Debug
{
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "Map({:?}, ...)", self.0)
    }
}

impl<P, F> Parser for Map<P, F> {}

impl<P, F, Ch, Str, Output> Stateful<Ch, Str, Output> for Map<P, F>
    where P: StatefulInfer<Ch, Str>,
          F: Function<P::Output, Output = Output>,
{

    fn done(self) -> Output {
        self.1.apply(self.0.done())
    }

    fn more(self, string: &mut Str) -> ParseResult<Self, Output> {
        match self.0.more(string) {
            Done(result) => Done(self.1.apply(result)),
            Continue(state) => Continue(Map(state, self.1)),
        }
    }

}

impl<P, F, Ch, Str> HasOutput<Ch, Str> for Map<P, F>
    where P: HasOutput<Ch, Str>,
          F: Function<P::Output>,
{

    type Output = F::Output;

}

impl<P, F, Ch, Str, Output> Committed<Ch, Str, Output> for Map<P, F>
    where P: CommittedInfer<Ch, Str>,
          F: 'static + Copy + Function<P::Output, Output = Output>,
{

    fn empty(&self) -> Output {
        self.1.apply(self.0.empty())
    }

}

impl<P, F, Ch, Str, Output> Uncommitted<Ch, Str, Output> for Map<P, F>
    where P: UncommittedInfer<Ch, Str>,
          F: 'static + Copy + Function<P::Output, Output = Output>,
{
    type State = Map<P::State, F>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Output>> {
        match self.0.init(string) {
            None => None,
            Some(Done(result)) => Some(Done(self.1.apply(result))),
            Some(Continue(state)) => Some(Continue(Map(state, self.1))),
        }
    }

}

impl<P, F> Map<P, F> {
    pub fn new(p: P, f: F) -> Self {
        Map(p, f)
    }
}

// ----------- Variant map ---------------

// A version of map for functions that can comute their input types from their output types

pub struct VariantMap<P, F>(P, F);

// A work around for functions implmenting copy but not clone
// https://github.com/rust-lang/rust/issues/28229
impl<P, F> Copy for VariantMap<P, F>
    where P: Copy,
          F: Copy
{}
impl<P, F> Clone for VariantMap<P, F>
    where P: Clone,
          F: Copy
{
    fn clone(&self) -> Self {
        VariantMap(self.0.clone(), self.1)
    }
}

// A work around for named functions not implmenting Debug
// https://github.com/rust-lang/rust/issues/31522
impl<P, F> Debug for VariantMap<P, F>
    where P: Debug
{
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "VariantMap({:?}, ...)", self.0)
    }
}

impl<P, F> Parser for VariantMap<P, F> {}

impl<P, F, Ch, Str, Output> Stateful<Ch, Str, Output> for VariantMap<P, F>
    where P: Stateful<Ch, Str, F::Input>,
          F: VariantFunction<Output>,
{

    fn done(self) -> Output {
        self.1.apply(self.0.done())
    }

    fn more(self, string: &mut Str) -> ParseResult<Self, Output> {
        match self.0.more(string) {
            Done(result) => Done(self.1.apply(result)),
            Continue(state) => Continue(VariantMap(state, self.1)),
        }
    }

}

impl<P, F, Ch, Str> HasOutput<Ch, Str> for VariantMap<P, F>
    where P: HasOutput<Ch, Str>,
          F: Function<P::Output>,
{

    type Output = F::Output;

}

impl<P, F, Ch, Str, Output> Committed<Ch, Str, Output> for VariantMap<P, F>
    where P: Committed<Ch, Str, F::Input>,
          F: 'static + Copy + VariantFunction<Output>,

{

    fn empty(&self) -> Output {
        self.1.apply(self.0.empty())
    }

}

impl<P, F, Ch, Str, Output> Uncommitted<Ch, Str, Output> for VariantMap<P, F>
    where P: Uncommitted<Ch, Str, F::Input>,
          F: 'static + Copy + VariantFunction<Output>,
{
    type State = VariantMap<P::State, F>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Output>> {
        match self.0.init(string) {
            None => None,
            Some(Done(result)) => Some(Done(self.1.apply(result))),
            Some(Continue(state)) => Some(Continue(VariantMap(state, self.1))),
        }
    }

}

impl<P, F> VariantMap<P, F> {
    pub fn new(p: P, f: F) -> Self {
        VariantMap(p, f)
    }
}

// ----------- Sequencing ---------------

#[derive(Copy, Clone, Debug)]
pub struct AndThen<P, Q>(P, Q);

impl<P, Q> Parser for AndThen<P, Q> {}

impl<P, Q, Ch, Str, POutput, PStaticOutput, QOutput> Committed<Ch, Str, (POutput, QOutput)> for AndThen<P, Q>
    where P: Committed<Ch, Str, POutput>,
          Q: 'static + Copy + Committed<Ch, Str, QOutput>,
          POutput: ToStatic<Static = PStaticOutput> + Downcast<PStaticOutput>,
{

    fn empty(&self) -> (POutput, QOutput) {
        (self.0.empty(), self.1.empty())
    }

}

impl<P, Q, Ch, Str, POutput, PStaticOutput, QOutput> Uncommitted<Ch, Str, (POutput, QOutput)> for AndThen<P, Q>
    where P: Uncommitted<Ch, Str, POutput>,
          Q: 'static + Copy + Committed<Ch, Str, QOutput>,
          POutput: ToStatic<Static = PStaticOutput> + Downcast<PStaticOutput>,
{

    type State = AndThenState<P::State, Q, PStaticOutput, Q::State>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, (POutput, QOutput)>> {
        match self.0.init(string) {
            None => None,
            Some(Done(fst)) => match self.1.init(string) {
                None => Some(Continue(InBetween(fst.downcast(), self.1))),
                Some(Done(snd)) => Some(Done((fst, snd))),
                Some(Continue(snd)) => Some(Continue(InRhs(fst.downcast(), snd))),
            },
            Some(Continue(fst)) => Some(Continue(InLhs(fst, self.1))),
        }
    }

}

impl<P, Q, Ch, Str> HasOutput<Ch, Str> for AndThen<P, Q>
    where P: HasOutput<Ch, Str>,
          Q: HasOutput<Ch, Str>,
{

    type Output = (P::Output, Q::Output);

}

impl<P, Q> AndThen<P, Q> {
    pub fn new(p: P, q: Q) -> Self {
        AndThen(p, q)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AndThenState<PState, Q, PStaticOutput, QState> {
    InLhs(PState, Q),
    InBetween(PStaticOutput, Q),
    InRhs(PStaticOutput, QState),
}

impl<PState, Q, PStaticOutput, QState, Ch, Str, POutput, QOutput> Stateful<Ch, Str, (POutput, QOutput)> for AndThenState<PState, Q, PStaticOutput, QState>
    where PState: Stateful<Ch, Str, POutput>,
          Q: Committed<Ch, Str, QOutput, State = QState>,
          QState: Stateful<Ch, Str, QOutput>,
          POutput: Downcast<PStaticOutput>,
          PStaticOutput: 'static + Upcast<POutput>,
{

    fn done(self) -> (POutput, QOutput)
    {
        match self {
            InLhs(fst, snd) => (fst.done(), snd.empty()),
            InBetween(fst, snd) => (fst.upcast(), snd.empty()),
            InRhs(fst, snd) => (fst.upcast(), snd.done()),
        }
    }

    fn more(self, string: &mut Str) -> ParseResult<Self, (POutput, QOutput)>
    {
        match self {
            InLhs(fst, snd) => {
                match fst.more(string) {
                    Done(fst) => match snd.init(string) {
                        None => Continue(InBetween(fst.downcast(), snd)),
                        Some(Done(snd)) => Done((fst, snd)),
                        Some(Continue(snd)) => Continue(InRhs(fst.downcast(), snd)),
                    },
                    Continue(fst) => Continue(InLhs(fst, snd)),
                }
            }
            InBetween(fst, snd) => {
                match snd.init(string) {
                    None => Continue(InBetween(fst, snd)),
                    Some(Done(snd)) => Done((fst.upcast(), snd)),
                    Some(Continue(snd)) => Continue(InRhs(fst, snd)),
                }
            }
            InRhs(fst, snd) => {
                match snd.more(string) {
                    Done(snd) => Done((fst.upcast(), snd)),
                    Continue(snd) => Continue(InRhs(fst, snd)),
                }
            }
        }
    }

}

impl<PState, Q, PStaticOutput, QState, Ch, Str> HasOutput<Ch, Str> for AndThenState<PState, Q, PStaticOutput, QState>
    where PState: HasOutput<Ch, Str>,
          Q: HasOutput<Ch, Str>,
{

    type Output = (PState::Output, Q::Output);

}

// ----------- Choice ---------------

#[derive(Copy, Clone, Debug)]
pub struct OrElse<P, Q>(P, Q);

impl<P, Q> Parser for OrElse<P, Q> {}

impl<P, Q, Ch, Str, Output> Committed<Ch, Str, Output> for OrElse<P, Q>
    where P: Uncommitted<Ch, Str, Output>,
          Q: Committed<Ch, Str, Output>,
{

    fn empty(&self) -> Output {
        self.1.empty()
    }

}

impl<P, Q, Ch, Str, Output> Uncommitted<Ch, Str, Output> for OrElse<P, Q>
    where P: Uncommitted<Ch, Str, Output>,
          Q: Uncommitted<Ch, Str, Output>,
{

    type State = OrElseState<P::State, Q::State>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Output>> {
        match self.0.init(string) {
            Some(Done(result)) => Some(Done(result)),
            Some(Continue(lhs)) => Some(Continue(Lhs(lhs))),
            None => match self.1.init(string) {
                Some(Done(result)) => Some(Done(result)),
                Some(Continue(rhs)) => Some(Continue(Rhs(rhs))),
                None => None,
            },
        }
    }

}

impl<P, Q, Ch, Str> HasOutput<Ch, Str> for OrElse<P, Q>
    where P: HasOutput<Ch, Str>,
{

    type Output = P::Output;

}

impl<P, Q> OrElse<P, Q> {
    pub fn new(lhs: P, rhs: Q) -> Self {
        OrElse(lhs, rhs)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum OrElseState<P, Q> {
    Lhs(P),
    Rhs(Q),
}

impl<P, Q, Ch, Str, Output> Stateful<Ch, Str, Output> for OrElseState<P, Q>
    where P: Stateful<Ch, Str, Output>,
          Q: Stateful<Ch, Str, Output>,
{
    fn more(self, string: &mut Str) -> ParseResult<Self, Output> {
        match self {
            Lhs(lhs) => match lhs.more(string) {
                Done(result) => Done(result),
                Continue(lhs) => Continue(Lhs(lhs)),
            },
            Rhs(rhs) => match rhs.more(string) {
                Done(result) => Done(result),
                Continue(rhs) => Continue(Rhs(rhs)),
            },
        }
    }

    fn done(self) -> Output {
        match self {
            Lhs(lhs) => lhs.done(),
            Rhs(rhs) => rhs.done(),
        }
    }

}

impl<P, Q, Ch, Str> HasOutput<Ch, Str> for OrElseState<P, Q>
    where P: HasOutput<Ch, Str>,
{
    type Output = P::Output;
}

// ----------- Kleene star ---------------

#[derive(Clone,Debug)]
pub struct StarState<P, PState, T>(P, Option<PState>, T);

impl<P, PState, T, Ch, Str> Stateful<Ch, Str, T> for StarState<P, PState, T>
    where P: Copy + UncommittedInfer<Ch, Str, State = PState>,
          PState: Stateful<Ch, Str, P::Output>,
          T: Consumer<P::Output>,
          Str: PeekableIterator,
{
    fn more(mut self, string: &mut Str) -> ParseResult<Self, T> {
        loop {
            match self.1.take() {
                None => {
                    match self.0.init(string) {
                        Some(Continue(state)) => return Continue(StarState(self.0, Some(state), self.2)),
                        Some(Done(result)) => self.2.accept(result),
                        None => return if string.is_empty() {
                            Continue(self)
                        } else {
                            Done(self.2)
                        },
                    }
                }
                Some(state) => {
                    match state.more(string) {
                        Continue(state) => return Continue(StarState(self.0, Some(state), self.2)),
                        Done(result) => self.2.accept(result),
                    }
                }
            }
        }
    }
    fn done(self) -> T {
        self.2
    }
}

impl<P, PState, T, Ch, Str> HasOutput<Ch, Str> for StarState<P, PState, T>
{
    type Output = T;
}

pub struct Plus<P, F>(P, F);

// A work around for functions implmenting copy but not clone
// https://github.com/rust-lang/rust/issues/28229
impl<P, F> Copy for Plus<P, F>
    where P: Copy,
          F: Copy
{}
impl<P, F> Clone for Plus<P, F>
    where P: Clone,
          F: Copy
{
    fn clone(&self) -> Self {
        Plus(self.0.clone(), self.1)
    }
}

// A work around for named functions not implmenting Debug
// https://github.com/rust-lang/rust/issues/31522
impl<P, F> Debug for Plus<P, F>
    where P: Debug
{
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "Plus({:?}, ...)", self.0)
    }
}

impl<P, F> Parser for Plus<P, F> {}

impl<P, F, Ch, Str> Uncommitted<Ch, Str, F::Output> for Plus<P, F>
    where P: 'static + Copy + UncommittedInfer<Ch, Str>,
          F: 'static + Factory,
          Str: PeekableIterator,
          P::State: Stateful<Ch, Str, <P as HasOutput<Ch, Str>>::Output>,
          F::Output: Consumer<P::Output>,
{
    type State = StarState<P, P::State, F::Output>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, F::Output>> {
        match self.0.init(string) {
            None => None,
            Some(Continue(state)) => Some(Continue(StarState(self.0, Some(state), self.1.build()))),
            Some(Done(result)) => {
                let mut buffer = self.1.build();
                buffer.accept(result);
                Some(StarState(self.0, None, buffer).more(string))
            },
        }
    }
}

impl<P, F, Ch, Str> HasOutput<Ch, Str> for Plus<P, F>
    where F: Factory,
{
    type Output = F::Output;
}

impl<P, F> Plus<P, F> {
    pub fn new(parser: P, factory: F) -> Self {
        Plus(parser, factory)
    }
}

pub struct Star<P, F>(P, F);

// A work around for functions implmenting copy but not clone
// https://github.com/rust-lang/rust/issues/28229
impl<P, F> Copy for Star<P, F>
    where P: Copy,
          F: Copy
{}
impl<P, F> Clone for Star<P, F>
    where P: Clone,
          F: Copy
{
    fn clone(&self) -> Self {
        Star(self.0.clone(), self.1)
    }
}

// A work around for named functions not implmenting Debug
// https://github.com/rust-lang/rust/issues/31522
impl<P, F> Debug for Star<P, F>
    where P: Debug
{
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "Star({:?}, ...)", self.0)
    }
}

impl<P, F> Parser for Star<P, F> {}

impl<P, F, Ch, Str> HasOutput<Ch, Str> for Star<P, F>
    where F: Factory,
{

    type Output = F::Output;

}

impl<P, F, Ch, Str> Uncommitted<Ch, Str, F::Output> for Star<P, F>
    where P: 'static + Copy + UncommittedInfer<Ch, Str>,
          F: 'static + Factory,
          Str: PeekableIterator,
          P::State: Stateful<Ch, Str, <P as HasOutput<Ch, Str>>::Output>,
          F::Output: Consumer<P::Output>,
{

    type State = StarState<P, P::State, F::Output>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, F::Output>> {
        if string.is_empty() {
            None
        } else {
            Some(StarState(self.0, None, self.1.build()).more(string))
        }
    }

}

impl<P, F, Ch, Str> Committed<Ch, Str, F::Output> for Star<P, F>
    where P: 'static + Copy + UncommittedInfer<Ch, Str>,
          F: 'static + Factory,
          Str: PeekableIterator,
          P::State: Stateful<Ch, Str, <P as HasOutput<Ch, Str>>::Output>,
          F::Output: Consumer<P::Output>,
{

    fn empty(&self) -> F::Output {
        self.1.build()
    }

}

impl<P, F> Star<P, F> {
    pub fn new(parser: P, factory: F) -> Self {
        Star(parser, factory)
    }
}

// ----------- Optional parse -------------

#[derive(Copy, Clone, Debug)]
pub struct Opt<P>(P);

impl<P> Parser for Opt<P> where P: Parser {}

impl<P, Ch, Str, Output> Stateful<Ch, Str, Option<Output>> for Opt<P>
    where P: Stateful<Ch, Str, Output>,
{

    fn more(self, string: &mut Str) -> ParseResult<Self, Option<Output>> {
        match self.0.more(string) {
            Done(result) => Done(Some(result)),
            Continue(parsing) => Continue(Opt(parsing)),
        }
    }

    fn done(self) -> Option<Output> {
        Some(self.0.done())
    }

}

impl<P, Ch, Str> HasOutput<Ch, Str> for Opt<P>
    where P: HasOutput<Ch, Str>,
{

    type Output = Option<P::Output>;

}

impl<P, Ch, Str, Output> Uncommitted<Ch, Str, Option<Output>> for Opt<P>
    where Str: PeekableIterator,
          P: Uncommitted<Ch, Str, Output>,
{

    type State = Opt<P::State>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Option<Output>>> {
        match self.0.init(string) {
            None => if string.is_empty() {
                None
            } else {
                Some(Done(None))
            },
            Some(Done(result)) => Some(Done(Some(result))),
            Some(Continue(parsing)) => Some(Continue(Opt(parsing))),
        }
    }

}

impl<P, Ch, Str, Output> Committed<Ch, Str, Option<Output>> for Opt<P>
    where Str: PeekableIterator,
          P: Uncommitted<Ch, Str, Output>,
{

    fn empty(&self) -> Option<Output>{
        None
    }

}

impl<P> Opt<P> {
    pub fn new(parser: P) -> Self {
        Opt(parser)
    }
}

// ----------- Parse but discard the result -------------

#[derive(Copy, Clone, Debug)]
pub struct Discard<P>(P);

impl<P> Parser for Discard<P> where P: Parser {}

impl<P, Ch, Str> Stateful<Ch, Str, ()> for Discard<P>
    where P: StatefulInfer<Ch, Str>,
{

    fn more(self, string: &mut Str) -> ParseResult<Self, ()> {
        match self.0.more(string) {
            Done(_) => Done(()),
            Continue(parsing) => Continue(Discard(parsing)),
        }
    }

    fn done(self) -> () {
        ()
    }

}

impl<P, Ch, Str> HasOutput<Ch, Str> for Discard<P>
{

    type Output = ();

}

impl<P, Ch, Str> Uncommitted<Ch, Str, ()> for Discard<P>
    where P: UncommittedInfer<Ch, Str>,
{

    type State = Discard<P::State>;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, ()>> {
        match self.0.init(string) {
            None => None,
            Some(Done(_)) => Some(Done(())),
            Some(Continue(parsing)) => Some(Continue(Discard(parsing))),
        }
    }

}

impl<P, Ch, Str> Committed<Ch, Str, ()> for Discard<P>
    where P: CommittedInfer<Ch, Str>,
{

    fn empty(&self) -> () {
        ()
    }

}

impl<P> Discard<P> {
    pub fn new(parser: P) -> Self {
        Discard(parser)
    }
}

// ----------- A type for parsers which immediately emit a result -------------

#[derive(Copy, Clone, Debug)]
pub struct Emit<F>(F);

impl<F> Parser for Emit<F> {}

impl<F, Ch, Str> Stateful<Ch, Str, F::Output> for Emit<F>
    where F: Factory,
{

    fn more(self, _: &mut Str) -> ParseResult<Self, F::Output> {
        Done(self.0.build())
    }

    fn done(self) -> F::Output {
        self.0.build()
    }

}

impl<F, Ch, Str> HasOutput<Ch, Str> for Emit<F>
    where F: Factory,
{

    type Output = F::Output;

}

impl<F, Ch, Str> Uncommitted<Ch, Str, F::Output> for Emit<F>
    where Str: PeekableIterator,
          F: 'static + Copy + Factory,
{

    type State = Self;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self, F::Output>> {
        if string.is_empty() {
            None
        } else {
            Some(Done(self.0.build()))
        }
    }

}

impl<F, Ch, Str> Committed<Ch, Str, F::Output> for Emit<F>
    where Str: PeekableIterator,
          F: 'static + Copy + Factory,
{

    fn empty(&self) -> F::Output {
        self.0.build()
    }
}

impl<T> Emit<T> {
    pub fn new(t: T) -> Self {
        Emit(t)
    }
}

// ----------- Character parsers -------------

#[derive(Copy, Clone, Debug)]
pub enum CharacterState {}

impl<Ch, Str> Stateful<Ch, Str, Ch> for CharacterState
{
    fn more(self, _: &mut Str) -> ParseResult<Self, Ch> { 
        match self {}
   }

    fn done(self) -> Ch {
        match self {}
    }
}

impl<Ch, Str> HasOutput<Ch, Str> for CharacterState
{
    type Output = Ch;
}

pub struct Character<F>(F);

// A work around for functions implmenting copy but not clone
// https://github.com/rust-lang/rust/issues/28229
impl<F> Copy for Character<F> where F: Copy {}
impl<F> Clone for Character<F> where F: Copy
{
    fn clone(&self) -> Self {
        Character(self.0)
    }
}

// A work around for named functions not implmenting Debug
// https://github.com/rust-lang/rust/issues/31522
impl<F> Debug for Character<F>
{
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "Character(...)")
    }
}

impl<F> Parser for Character<F> {}

impl<F, Ch, Str> HasOutput<Ch, Str> for Character<F>
{
    type Output = Ch;
}

impl<F, Ch, Str> Uncommitted<Ch, Str, Ch> for Character<F>
    where Str: PeekableIterator<Item = Ch>,
          F: Copy + Function<Ch, Output = bool>,
          Ch: Copy,
{
    type State = CharacterState;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Ch>> {
        match string.next_if(self.0) {
            None => None,
            Some(ch) => Some(Done(ch)),
        }
    }

}

impl<F> Character<F> {
    pub fn new(function: F) -> Self {
        Character(function)
    }
}

pub struct CharacterRef<F>(F);

// A work around for functions implmenting copy but not clone
// https://github.com/rust-lang/rust/issues/28229
impl<F> Copy for CharacterRef<F> where F: Copy {}
impl<F> Clone for CharacterRef<F> where F: Copy
{
    fn clone(&self) -> Self {
        CharacterRef(self.0)
    }
}

// A work around for named functions not implmenting Debug
// https://github.com/rust-lang/rust/issues/31522
impl<F> Debug for CharacterRef<F>
{
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "CharacterRef(...)")
    }
}

impl<F> Parser for CharacterRef<F> {}

impl<F, Ch, Str> HasOutput<Ch, Str> for CharacterRef<F>
{
    type Output = Ch;
}

impl<F, Ch, Str> Uncommitted<Ch, Str, Ch> for CharacterRef<F>
    where Str: PeekableIterator<Item = Ch>,
          F: Copy + for<'a> Function<&'a Ch, Output = bool>,
{
    type State = CharacterState;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Ch>> {
        match string.next_if_ref(self.0) {
            None => None,
            Some(ch) => Some(Done(ch)),
        }
    }

}

impl<F> CharacterRef<F> {
    pub fn new(function: F) -> Self {
        CharacterRef(function)
    }
}

#[derive(Copy,Clone,Debug)]
pub struct AnyCharacter;

impl Parser for AnyCharacter {}

impl<Ch, Str> Stateful<Ch, Str, Option<Ch>> for AnyCharacter
    where Str: Iterator<Item = Ch>,
{
    fn more(self, string: &mut Str) -> ParseResult<Self, Option<Ch>> {
        match string.next() {
            None => Continue(self),
            Some(ch) => Done(Some(ch)),
        }
    }

    fn done(self) -> Option<Ch> {
        None
    }
}

impl<Ch, Str> HasOutput<Ch, Str> for AnyCharacter
    where Str: Iterator<Item = Ch>,
{
    type Output = Option<Ch>;
}

impl<Ch, Str> Uncommitted<Ch, Str, Option<Ch>> for AnyCharacter
    where Str: Iterator<Item = Ch>,
{
    type State = AnyCharacter;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Option<Ch>>> {
        match string.next() {
            None => None,
            Some(ch) => Some(Done(Some(ch))),
        }
    }

}

impl<Ch, Str> Committed<Ch, Str, Option<Ch>> for AnyCharacter
    where Str: Iterator<Item = Ch>,
{

    fn empty(&self) -> Option<Ch> {
        None
    }

}

// ----------- Buffering -------------

// If p is a UncommittedInfer<char, Chars<'a>>, then
// m.buffer() is a UncommittedInfer<char, Chars<'a>> with Output (char, Cow<'a,str>).
// It does as little buffering as it can, but it does allocate as buffer for the case
// where the boundary marker of the input is misaligned with that of the parser.
// For example, m is matching string literals, and the input is '"abc' followed by 'def"'
// we have to buffer up '"abc'.

// TODO(ajeffrey): make this code generic.

#[derive(Copy, Clone, Debug)]
pub struct Buffered<P>(P);

impl<P> Parser for Buffered<P> where P: Parser {}

impl<'a, P> HasOutput<char, Chars<'a>> for Buffered<P>
{
    type Output = Cow<'a, str>;
}

impl<'a, P> Uncommitted<char, Chars<'a>, Cow<'a, str>> for Buffered<P>
    where P: UncommittedInfer<char, Chars<'a>>,
{
    type State = BufferedState<P::State>;

    fn init(&self, string: &mut Chars<'a>) -> Option<ParseResult<Self::State, Cow<'a, str>>> {
        let string0 = string.as_str();
        match self.0.init(string) {
            Some(Done(_)) => Some(Done(Borrowed(&string0[..(string0.len() - string.as_str().len())]))),
            Some(Continue(state)) => Some(Continue(BufferedState(state, String::from(string0)))),
            None => None,
        }
    }
}

impl<'a, P> Committed<char, Chars<'a>, Cow<'a, str>> for Buffered<P>
    where P: CommittedInfer<char, Chars<'a>>,
{
    fn empty(&self) -> Cow<'a, str> { Borrowed("") }
}

impl<P> Buffered<P> {
    pub fn new(parser: P) -> Self {
        Buffered(parser)
    }
}

#[derive(Clone,Debug)]
pub struct BufferedState<P>(P, String);

impl<'a, P> Stateful<char, Chars<'a>, Cow<'a, str>> for BufferedState<P>
    where P: StatefulInfer<char, Chars<'a>>
{

    fn more(mut self, string: &mut Chars<'a>) -> ParseResult<Self, Cow<'a, str>> {
        let string0 = string.as_str();
        match self.0.more(string) {
            Done(_) => {
                self.1.push_str(&string0[..(string0.len() - string.as_str().len())]);
                Done(Owned(self.1))
            },
            Continue(state) => {
                self.1.push_str(string0);
                Continue(BufferedState(state, self.1))
            },
        }
    }

    fn done(self) -> Cow<'a, str> {
        Owned(self.1)
    }

}

impl<'a, P> HasOutput<char, Chars<'a>> for BufferedState<P>
    where P: HasOutput<char, Chars<'a>>
{

    type Output = Cow<'a, str>;

}

// ----------- Parsers which are boxable -------------

#[derive(Debug)]
pub struct BoxableState<P>(Option<P>);

impl<P, Ch, Str, Output> Boxable<Ch, Str, Output> for BoxableState<P>
    where P: Stateful<Ch, Str, Output>,
{
    fn more_boxable(&mut self, string: &mut Str) -> ParseResult<(), Output> {
        match self.0.take().unwrap().more(string) {
            Done(result) => Done(result),
            Continue(state) => {
                self.0 = Some(state);
                Continue(())
            }
        }
    }
    fn done_boxable(&mut self) -> Output {
        self.0.take().unwrap().done()
    }
}

impl<P, Ch, Str> HasOutput<Ch, Str> for BoxableState<P>
    where P: HasOutput<Ch, Str>,
{
    type Output = P::Output;
}

impl<P: ?Sized, Ch, Str, Output> Stateful<Ch, Str, Output> for Box<P>
    where P: Boxable<Ch, Str, Output>,
{
    fn more(mut self, string: &mut Str) -> ParseResult<Self, Output> {
        match self.more_boxable(string) {
            Done(result) => Done(result),
            Continue(()) => Continue(self),
        }
    }
    fn done(mut self) -> Output {
        self.done_boxable()
    }
}

impl<P> BoxableState<P> {
    pub fn new(parser: P) -> Self {
        BoxableState(Some(parser))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Boxed<P, F>(P, F);

impl<P, F> Parser for Boxed<P, F> where P: Parser {}

impl<P, F, Ch, Str> HasOutput<Ch, Str> for Boxed<P, F>
    where P: HasOutput<Ch, Str>,
{
    type Output = P::Output;
}

impl<P, F, Ch, Str, Output> Uncommitted<Ch, Str, Output> for Boxed<P, F>
    where P: Uncommitted<Ch, Str, Output>,
          F: Function<BoxableState<P::State>>,
          F::Output: 'static + Stateful<Ch, Str, Output>,
{
    type State = F::Output;

    fn init(&self, string: &mut Str) -> Option<ParseResult<Self::State, Output>> {
        match self.0.init(string) {
            None => None,
            Some(Done(result)) => Some(Done(result)),
            Some(Continue(parsing)) => Some(Continue(self.1.apply(BoxableState::new(parsing)))),
        }
    }
}

impl<P, F, Ch, Str, Output> Committed<Ch, Str, Output> for Boxed<P, F>
    where P: Committed<Ch, Str, Output>,
          F: Function<BoxableState<P::State>>,
          F::Output: 'static + Stateful<Ch, Str, Output>,
{
    fn empty(&self) -> Output {
        self.0.empty()
    }
}

impl<P, F> Boxed<P, F> {
    pub fn new(parser: P, function: F) -> Self {
        Boxed(parser, function)
    }
}

// // ----------- Iterate over parse results -------------

// #[derive(Copy, Clone, Debug)]
// pub struct IterParser<P, Q, S>(P, Option<(Q, S)>);

// impl<P, Str> Iterator for IterParser<P, P::State, Str>
//     where P: Copy + CommittedInfer<Str>,
//           Str: IntoPeekable,
//           Str::Item: ToStatic,
//           P::State: StatefulInfer<Str>,
// {
//     type Item = <P::State as StatefulInfer<Str>>::Output;
//     fn next(&mut self) -> Option<Self::Item> {
//         let (state, result) = match self.1.take() {
//             None => (None, None),
//             Some((parsing, data)) => {
//                 match parsing.parse(data) {
//                     Done(rest, result) => (Some((self.0.init(), rest)), Some(result)),
//                     Continue(rest, parsing) => (Some((parsing, rest)), None),
//                 }
//             }
//         };
//         *self = IterParser(self.0, state);
//         result
//     }
// }

// impl<P, Str> IterParser<P, P::State, Str>
//     where P: Copy + CommittedInfer<Str>,
//           Str: IntoPeekable,
//           Str::Item: ToStatic,
// {
//     pub fn new(parser: P, data: Str) -> Self {
//         IterParser(parser, Some((parser.init(), data)))
//     }
// }

// // ----------- Pipe parsers -------------

// TODO: restore these

// #[derive(Copy, Clone, Debug)]
// pub struct PipeStatefulInfer<P, Q, R>(P, Q, R);

// impl<P, Q, Str> StatefulInfer<Str> for PipeStatefulInfer<P, P::State, Q>
//     where P: Copy + CommittedInfer<Str>,
//           Q: StatefulInfer<Peekable<IterParser<P, P::State, Str>>>,
//           Str: IntoPeekable,
//           Str::Item: ToStatic,
//           P::State: StatefulInfer<Str>,
// {
//     type Output = Q::Output;
//     fn parse(self, data: Str) -> ParseResult<Self, Str> {
//         let iterator = Peekable::new(IterParser(self.0, Some((self.1, data))));
//         match self.2.parse(iterator) {
//             Done(rest, result) => Done(rest.iter.1.unwrap().1, result),
//             Continue(rest, parsing2) => {
//                 let (parsing1, data) = rest.iter.1.unwrap();
//                 Continue(data, PipeStatefulInfer(self.0, parsing1, parsing2))
//             }
//         }
//     }
//     fn done(self) -> Q::Output {
//         // TODO: feed the output of self.1.done() into self.2.
//         self.1.done();
//         self.2.done()
//     }
// }

// #[derive(Copy, Clone, Debug)]
// pub struct PipeParser<P, Q>(P, Q);

// impl<P, Q, Ch> Parser<Ch> for PipeParser<P, Q>
//     where P: 'static + Parser<Ch>,
//           Q: Parser<Ch>,
// {
//     type State = PipeStatefulInfer<P,P::State,Q::State>;
//     type StaticOutput = Q::StaticOutput;
// }

// impl<P, Q, Str> CommittedInfer<Str> for PipeParser<P, Q>
//     where P: 'static + Copy + CommittedInfer<Str>,
//           Q: for<'a> CommittedInfer<Peekable<&'a mut IterParser<P, P::State, Str>>>,
//           Str: IntoPeekable,
//           Str::Item: ToStatic,
//           P::State: StatefulInfer<Str>,
// {
//     fn init(&self) -> Self::State {
//         PipeStatefulInfer(self.0, self.0.init(), self.1.init())
//     }
// }

// impl<P, Q> PipeParser<P, Q> {
//     pub fn new(lhs: P, rhs: Q) -> Self {
//         PipeParser(lhs, rhs)
//     }
// }

