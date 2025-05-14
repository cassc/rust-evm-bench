package main

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/tests"
	"github.com/urfave/cli/v2"
)

func runBenchmark(iterations int, testFile string) {
	src, err := os.ReadFile(testFile)
	if err != nil {
		panic(fmt.Errorf("failed to read test file: %v", err))
	}

	var testsByName map[string]tests.StateTest
	if err := json.Unmarshal(src, &testsByName); err != nil {
		panic(fmt.Errorf("failed to unmarshal test data: %v", err))
	}

	for _, test := range testsByName {
		st := tests.StateSubtest{Fork: "Shanghai"}
		cfg := vm.Config{}

		// Warm-up iteration
		_, _, _, err = test.RunNoVerify(st, cfg, false, rawdb.HashScheme)
		if err != nil {
			panic(fmt.Errorf("warm-up failed: %v", err))
		}

		start := time.Now()
		for range iterations {
			_, _, _, err = test.RunNoVerify(st, cfg, false, rawdb.HashScheme)
			if err != nil {
				panic(fmt.Errorf("test execution failed: %v", err))
			}
		}
		duration := time.Since(start)

		fmt.Printf("Benchmark %s\t%.2f ms / transaction\n", testFile, float64(duration.Nanoseconds())/float64(iterations)/1000000.0)
		break // Only run first test
	}
}

func main() {
	app := &cli.App{
		Name:  "benchmark",
		Usage: "Run blockchain transaction benchmarks",
		Flags: []cli.Flag{
			&cli.IntFlag{
				Name:    "iterations",
				Aliases: []string{"n"},
				Value:   10000,
				Usage:   "Number of iterations to run",
			},
		},
		Action: func(c *cli.Context) error {
			if c.NArg() < 1 {
				return cli.Exit("Error: input JSON file path is required", 1)
			}
			jsonPath := c.Args().First()
			iterations := c.Int("iterations")

			fmt.Printf("Running %d transactions for each test case\n", iterations)
			runBenchmark(iterations, jsonPath)
			return nil
		},
	}

	if err := app.Run(os.Args); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}
