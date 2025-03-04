package main

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/tracing"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/triedb"
	"github.com/holiman/uint256"
)

type stEnv struct {
	Coinbase      common.Address        `json:"currentCoinbase"      gencodec:"required"`
	Difficulty    *math.HexOrDecimal256 `json:"currentDifficulty"    gencodec:"optional"`
	Random        *math.HexOrDecimal256 `json:"currentRandom"        gencodec:"optional"`
	GasLimit      uint64                `json:"currentGasLimit"      gencodec:"required"`
	Number        uint64                `json:"currentNumber"        gencodec:"required"`
	Timestamp     uint64                `json:"currentTimestamp"     gencodec:"required"`
	BaseFee       *math.HexOrDecimal256 `json:"currentBaseFee"       gencodec:"optional"`
	ExcessBlobGas *uint64               `json:"currentExcessBlobGas" gencodec:"optional"`
}

type stAuthorization struct {
	ChainID *math.HexOrDecimal256 `json:"chainId" gencodec:"required"`
	Address common.Address        `json:"address" gencodec:"required"`
	Nonce   uint64                `json:"nonce" gencodec:"required"`
	V       uint8                 `json:"v" gencodec:"required"`
	R       *math.HexOrDecimal256 `json:"r" gencodec:"required"`
	S       *math.HexOrDecimal256 `json:"s" gencodec:"required"`
}

type stTransaction struct {
	GasPrice             *math.HexOrDecimal256 `json:"gasPrice"`
	MaxFeePerGas         *math.HexOrDecimal256 `json:"maxFeePerGas"`
	MaxPriorityFeePerGas *math.HexOrDecimal256 `json:"maxPriorityFeePerGas"`
	Nonce                uint64                `json:"nonce"`
	To                   string                `json:"to"`
	Data                 []string              `json:"data"`
	AccessLists          []*types.AccessList   `json:"accessLists,omitempty"`
	GasLimit             []uint64              `json:"gasLimit"`
	Value                []string              `json:"value"`
	PrivateKey           []byte                `json:"secretKey"`
	Sender               *common.Address       `json:"sender"`
	BlobVersionedHashes  []common.Hash         `json:"blobVersionedHashes,omitempty"`
	BlobGasFeeCap        *math.HexOrDecimal256 `json:"maxFeePerBlobGas,omitempty"`
	AuthorizationList    []*stAuthorization    `json:"authorizationList,omitempty"`
}

type StateTest struct {
	Env stEnv              `json:"env"`
	Pre types.GenesisAlloc `json:"pre"`
	Tx  stTransaction      `json:"transaction"`
	Out hexutil.Bytes      `json:"out"`
}

func benchmark_file(fname string) {
	src, err := os.ReadFile(fname)
	if err != nil {
		panic("read file content failed")
	}

	var testsByName map[string]StateTest

	if err := json.Unmarshal(src, &testsByName); err != nil {
		panic(fmt.Errorf("unable to read test file %s: %w", fname, err))
	}

	fmt.Printf("Parsed content: %v\n", testsByName)

	for _, test := range testsByName {

		// Get tx
		to := common.HexToAddress(test.Tx.To)
		from := test.Tx.Sender
		// value from json.Tx.value[0]
		value := new(big.Int)
		value, ok := value.SetString(test.Tx.Value[0], 16)
		if !ok {
			panic("Invalid value")
		}

		gasLimit := test.Tx.GasLimit[0]
		gasPrice := test.Tx.GasPrice
		msgData := common.FromHex(test.Tx.Data[0])

		msg := &core.Message{
			To:       &to,
			From:     *from,
			Nonce:    test.Tx.Nonce,
			Value:    value,
			GasLimit: gasLimit,
			GasPrice: gasPrice,
			Data:     msgData,
		}

		tconf := &triedb.Config{Preimages: true}
		triedb := triedb.NewDatabase(rawdb.NewMemoryDatabase(), tconf)
		sdb := state.NewDatabase(triedb, nil)
		stateDB, err := state.New(common.Hash{}, sdb)
		if err != nil {
			panic(fmt.Errorf("Error creating state database: %v\n", err))
		}

		// load states from json
		for address, account := range test.Pre {
			stateDB.SetBalance(address, uint256.MustFromBig(account.Balance), tracing.BalanceChangeTouchAccount)
			stateDB.SetNonce(address, account.Nonce, tracing.NonceChangeAuthorization)
			stateDB.SetCode(address, account.Code)
			// load storage key-values
			for key, value := range account.Storage {
				stateDB.SetState(address, key, value)
			}
		}

		// block context

		blockContext := vm.BlockContext{
			Transfer:    core.Transfer,
			GetHash:     func(n uint64) common.Hash { return common.Hash{} },
			Coinbase:    common.Address{},
			BlockNumber: big.NewInt(9069001), // Set to >= IstanbulBlock (9069000)
			Time:        test.Env.Timestamp,
			Difficulty:  test.Env.Difficulty,
			GasLimit:    test.Env.GasLimit,
			BaseFee:     test.Env.BaseFee,
		}

		chainConfig := params.MainnetChainConfig
		vmConfig := vm.Config{}
		evm := vm.NewEVM(blockContext, stateDB, chainConfig, vmConfig)

		_, _, err = evm.Call(msg.From, *msg.To, msg.Data, msg.GasLimit, uint256.MustFromBig(msg.Value))
		if err != nil {
			panic(fmt.Errorf("Error calling EVM: %v\n", err))
		}
	}

}

func main() {
	benchmark_file("../uniswap.bench.input.json")
}
