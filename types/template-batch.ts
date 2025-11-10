export interface TeraTemplateAnalysis {
	variables: string[]
	variableCount: number
	hasLoops: boolean
	hasConditionals: boolean
}

export interface TemplateExcelPreview {
	sheetNames: string[]
	selectedSheet: string
	headerRowIndex: number
	columns: string[]
	previewRows: string[][]
	totalRows: number
}

export interface GenericGeneratedConfig {
	label: string
	config: string
	rowIndex: number
}
