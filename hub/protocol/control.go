package protocol

type ControlFlag byte

const (
	Init       = 0xc0
	Code       = 0xc1
	Stdin      = 0xe1
	Stdout     = 0xe2
	Stderr     = 0xe3
	UsageInfo  = 0xe6
	StatusInfo = 0xe7
	ExitInfo   = 0xe8
)
