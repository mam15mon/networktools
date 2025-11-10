export interface ExcelAnalysis {
	sheetNames: string[]
	selectedSheet: string
	headerRowIndex: number
	columns: string[]
	previewRows: string[][]
	suggestedMapping: Record<string, string>
	totalRows: number
}

export interface NatEntry {
	rowIndex: number
	protocol: "TCP" | "UDP" | "ICMP" | "ANY"
	internalIp: string
	internalPortStart: number | null
	internalPortEnd: number | null
	publicIps: string[]
	publicPortStart: number | null
	publicPortEnd: number | null
	isPortRange: boolean
}

export interface ConvertResponse {
	entries: NatEntry[]
	errors: string[]
}

export type DeviceType = "huawei" | "h3c";

export type IspSource = "local" | "online"

export interface GenerateNatCommandsRequest {
	entries: NatEntry[]
	useElasticIp: boolean
	deviceType: DeviceType
	vrrpId?: number | null
	ispSource?: IspSource
}

export interface GenerateNatCommandsResponse {
	commands: string[]
	missingElasticIps: string[]
}

export interface ElasticMappingEntry {
	internalIp: string
	elasticIp: string
}

export interface BulkElasticResult {
	added: number
	updated: number
	skipped: number
}

export interface IspUpdateResult {
	dxCount: number
	ltCount: number
	ydCount: number
	otherCount: number
	total: number
	savedPath: string
}

export interface ManualEntryRequest {
	protocol: string
	internalIp: string
	internalPort?: string
	publicIp: string
	publicPort?: string
}
