export interface ExcelAnalysis {
	sheetNames: string[]
	selectedSheet: string
	headerRowIndex: number
	columns: string[]
	previewRows: string[][]
	suggestedMapping: Record<string, string>
	totalRows: number
}

export interface VsrEntry {
	deviceName: string
	ip: string
	gateway: string
	vsrUsername?: string
	vsrPassword?: string
	monitorUsername?: string
	monitorPassword?: string
	pppUsername?: string
	pppPassword?: string
	startIp: string
	endIp: string
	poolIpGateway: string
	ldapServerIp?: string
	ldapLoginDn?: string
	ldapSearchBaseDn?: string
	ldapPassword?: string
	radiusIp?: string
	radiusPassword?: string
	rowIndex: number
}

export interface ConvertResponse {
	entries: VsrEntry[]
	errors: string[]
}

export interface VsrGeneratedConfig {
	deviceName: string
	config: string
}
