package app

import "github.com/spf13/cobra"

type ProgramConfig struct {
	WindowDuration         int
	Spans                  int
	MinEvalSize            int
	ErrorThreshold         float64
	RetryTimeout           int
	TrialSuccessesRequired int
}

func Cmd() *cobra.Command {
	config := ProgramConfig{
		WindowDuration:         10,
		Spans:                  10,
		MinEvalSize:            100,
		ErrorThreshold:         10.0,
		RetryTimeout:           100,
		TrialSuccessesRequired: 20,
	}

	cmd := &cobra.Command{
		Use:   "vis [flags]",
		Short: "Circuit Breaker visualisation and testing tool",

		Run: func(cmd *cobra.Command, args []string) {
			runNewProgram(&config)
		},
	}

	cmd.Flags().IntVarP(&config.WindowDuration, "duration", "d", 10, "The total duration of the circuit breaker evaluation window in minutes. Defaults to 10 minutes.")
	cmd.Flags().IntVarP(&config.Spans, "spans", "s", 10, "The number of time spans the evaluation window is divided into. This allows for more granular data collection and analysis. Defaults to 10.")
	cmd.Flags().IntVarP(&config.MinEvalSize, "min-eval-size", "m", 100, "The minimum number of events required within the evaluation window to assess the error rate and determine the circuit breaker state. Defaults to 100.")
	cmd.Flags().Float64VarP(&config.ErrorThreshold, "error-threshold", "e", 10.0, "The error rate threshold that will cause the circuit breaker to open. Defaults to 10.0.")
	cmd.Flags().IntVarP(&config.RetryTimeout, "retry", "r", 100, "The duration in seconds the circuit breaker remains in the Open state before transitioning to Half-Open. Defaults to 60.")
	cmd.Flags().IntVarP(&config.TrialSuccessesRequired, "trials", "t", 20, "The number of consecutive successful requests needed while in the Half-Open state before the circuit breaker transitions back to the Closed state. Defaults to 20.")

	return cmd
}
