package main

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/tests"
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
	const iterations = 10000
	fmt.Printf("Running %d transactions for each test case\n", iterations)
	runBenchmark(iterations, "../erc20.bench.input.json")
	runBenchmark(iterations, "../uniswap.bench.input.json")
}
