app "closure"
    packages { base: "platform" }
    imports [base.Task]
    provides [ main ] to base

# see https://github.com/rtfeldman/roc/issues/985

main : Task.Task {} []
main = closure1 {}
    |> Task.after (\_ -> closure2 {})
    |> Task.after (\_ -> closure2 {})
    |> Task.after (\_ -> closure2 {})

# ---

closure1 : {} -> Task.Task {} []
closure1 = \_ ->
    Task.succeed (foo toUnitBorrowed "a long string such that it's malloced")
        |> Task.map (\_ -> {})

toUnitBorrowed = \x -> Str.countGraphemes x

foo = \f, x -> f x

# ---

closure2 : {} -> Task.Task {} []
closure2 = \_ ->
    x : Str
    x = "a long string such that it's malloced"

    Task.succeed {}
        |> Task.map (\_ -> x)
        |> Task.map toUnit

toUnit = (\_ -> {})

# ---

closure3 : {} -> Task.Task {} []
closure3 = \_ ->
    x : Str
    x = "a long string such that it's malloced"

    Task.succeed {}
        |> Task.after (\_ -> Task.succeed x |> Task.map (\_ -> {}))

# ---

closure4 : {} -> Task.Task {} []
closure4 = \_ ->
    x : Str
    x = "a long string such that it's malloced"

    Task.succeed {}
        |> Task.after (\_ -> Task.succeed x)
        |> Task.map (\_ -> {})
