package main

import (
	"fmt"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/tracing"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/triedb"
	"github.com/holiman/uint256"
)

const (
	ownerAddr        = "0xf000000000000000000000000000000000000000"
	contractAddr     = "0x0000000000000000000000000000000000000000"
	contractBin      = "608060405234801561001057600080fd5b506004361061002b5760003560e01c806302067e6a14610030575b600080fd5b61004a600480360381019061004591906100dd565b610060565b6040516100579190610175565b60405180910390f35b600060808260ff16106100a8576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161009f90610155565b60405180910390fd5b8160008054906101000a900460ff166100c191906101a1565b9050919050565b6000813590506100d781610214565b92915050565b6000602082840312156100ef57600080fd5b60006100fd848285016100c8565b91505092915050565b6000610113600e83610190565b91507f6e20697320746f6f206c617267650000000000000000000000000000000000006000830152602082019050919050565b61014f816101d8565b82525050565b6000602082019050818103600083015261016e81610106565b9050919050565b600060208201905061018a6000830184610146565b92915050565b600082825260208201905092915050565b60006101ac826101d8565b91506101b7836101d8565b92508260ff038211156101cd576101cc6101e5565b5b828201905092915050565b600060ff82169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b61021d816101d8565b811461022857600080fd5b5056fea2646970667358221220adf34242ee77b97aa044e069ebc15c434c1296636f349e726a9678ab019ab2f264736f6c63430008000033"
	methodSuccessBin = "02067e6a0000000000000000000000000000000000000000000000000000000000000001"
	methodRevertBin  = "02067e6a00000000000000000000000000000000000000000000000000000000000000c7"
)

func vmTestBlockHash(n uint64) common.Hash {
	return common.BytesToHash(crypto.Keccak256([]byte(big.NewInt(int64(n)).String())))
}

func runContractBenchmark(iterations int, testType string) {
	owner := common.HexToAddress(ownerAddr)
	contractAddr := common.HexToAddress(contractAddr)
	tconf := &triedb.Config{Preimages: true}
	triedb := triedb.NewDatabase(rawdb.NewMemoryDatabase(), tconf)
	sdb := state.NewDatabase(triedb, nil)

	stateDB, err := state.New(common.Hash{}, sdb)
	if err != nil {
		fmt.Printf("Error creating state database: %v\n", err)
		return
	}

	contractCode := common.FromHex(contractBin)

	stateDB.SetCode(contractAddr, contractCode)
	stateDB.AddBalance(owner, uint256.NewInt(1000000000000000000), tracing.BalanceChangeTouchAccount)

	successData := common.FromHex(methodSuccessBin)
	revertData := common.FromHex(methodRevertBin)
	var msgData []byte
	if testType == "success" {
		msgData = successData
	} else {
		msgData = revertData
	}

	msg := &core.Message{
		To:       &contractAddr,
		From:     owner,
		Nonce:    0,
		Value:    big.NewInt(0),
		GasLimit: 1000000,
		GasPrice: big.NewInt(20000000000),
		Data:     msgData,
	}

	blockContext := vm.BlockContext{
		CanTransfer: core.CanTransfer,
		Transfer:    core.Transfer,
		GetHash:     func(n uint64) common.Hash { return common.Hash{} },
		Coinbase:    common.Address{},
		BlockNumber: big.NewInt(9069001), // Set to >= IstanbulBlock (9069000)
		Time:        uint64(time.Now().Unix()),
		Difficulty:  big.NewInt(1),
		GasLimit:    10000000,
		BaseFee:     big.NewInt(0x0a),
	}

	blockContext.GetHash = vmTestBlockHash
	chainConfig := params.MainnetChainConfig
	vmConfig := vm.Config{}
	evm := vm.NewEVM(blockContext, stateDB, chainConfig, vmConfig)

	start := time.Now()
	for range iterations {
		snapshot := stateDB.Snapshot()

		_, _, err := evm.Call(msg.From, *msg.To, msg.Data, msg.GasLimit, uint256.MustFromBig(msg.Value))

		if testType == "success" {
			if err != nil {
				panic(fmt.Sprintf("Expected success but got error: %v", err))
			}
		} else if testType == "revert" {
			if err == nil {
				panic("Expected revert but transaction succeeded")
			}
			// Check if the error message is exactly "execution reverted"
			if err.Error() != "execution reverted" {
				panic(fmt.Sprintf("Expected error message 'execution reverted' but got: %v", err))
			}
		} else {
			panic(fmt.Sprintf("Invalid testType %v", testType))
		}
		stateDB.RevertToSnapshot(snapshot)
	}

	duration := time.Since(start)
	fmt.Printf("Ran %d iterations of %s test in %v\n", iterations, testType, duration)
	fmt.Printf("Average time per iteration: %v\n", duration/time.Duration(iterations))
}

func main() {
	iterations := 10000
	fmt.Printf("Starting Go EVM benchmark with %d iterations...\n", iterations)
	runContractBenchmark(iterations, "success")
	runContractBenchmark(iterations, "revert")
}
