import { computed, reactive, ref, watch } from "vue";
import type {
	GenericGeneratedConfig,
	TemplateExcelPreview,
	TeraTemplateAnalysis
} from "~/types/template-batch";
import { extractErrorMessage } from "~/utils/error";
import { buildSampleValue, formatExampleList } from "~/utils/templateHints";

const FORMATTING_FILTERS = [
	"upper",
	"lower",
	"capitalize",
	"title",
	"trim",
	"trim_end",
	"trim_start",
	"slice",
	"replace",
	"escape"
] as const;

const FORMATTING_DESCRIPTIONS: Record<string, string> = {
	upper: "自动转为大写",
	lower: "自动转为小写",
	capitalize: "首字母大写",
	title: "每个单词首字母大写",
	trim: "移除首尾空格",
	trim_end: "移除结尾空格",
	trim_start: "移除开头空格",
	slice: "截取字符串片段",
	replace: "替换字符",
	escape: "自动转义特殊字符"
};

export interface TemplateVariableInsight {
	name: string
	sample: string
	tags: {
		loop: boolean
		conditional: boolean
		defaultable: boolean
		formatting: boolean
	}
	formattingDescriptions: string[]
	defaultFallback?: string
	conditionalValues: string[]
}

export function useTemplateBatch() {
	const toast = useToast();

	const templateState = reactive({
		filePath: "",
		analysis: null as TeraTemplateAnalysis | null,
		isLoading: false
	});

	const excelState = reactive({
		filePath: "",
		preview: null as TemplateExcelPreview | null,
		selectedSheet: "",
		isLoading: false
	});

	const labelField = ref("");

	watch(
		() => templateState.analysis?.variables,
		(variables) => {
			if (variables && variables.length > 0) {
				labelField.value = variables[0];
			}
		},
		{ immediate: true }
	);

	const generationLoading = ref(false);
	const generationErrors = ref<string[]>([]);
	const generatedConfigs = ref<GenericGeneratedConfig[]>([]);
	const expandedConfigs = ref<Record<number, boolean>>({});
	const showAllConfigs = ref(false);

	const excelPreviewColumns = computed(() => {
		if (!excelState.preview) return [];
		return excelState.preview.columns.map((column, index) => ({
			id: `col_${index}`,
			accessorKey: `col_${index}`,
			header: column || `列 ${index + 1}`
		}));
	});

	const excelPreviewRows = computed(() => {
		if (!excelState.preview) return [];
		const previewLimit = 10;
		return excelState.preview.previewRows.slice(0, previewLimit).map((row) => {
			const rowObj: Record<string, string> = {};
			row.forEach((cell, index) => {
				rowObj[`col_${index}`] = cell;
			});
			return rowObj;
		});
	});

	const conditionalVariableSet = computed(() => {
		const map = templateState.analysis?.sampleValues;
		if (!map) return new Set<string>();
		const entries = Object.entries(map).filter(([, values]) => values && values.length > 0);
		return new Set(entries.map(([key]) => key));
	});

	const columnValidationStatus = computed(() => {
		if (!templateState.analysis || !excelState.preview) {
			return {
				isValid: false,
				missingVariables: [] as string[],
				emptyVariables: [] as string[],
				invalidIterableVariables: [] as string[]
			};
		}

		const requiredVariables = templateState.analysis.variables;
		const availableColumns = excelState.preview.columns;
		const columnsWithData = excelState.preview.columnsWithData || [];
		const invalidIterableColumns = excelState.preview.invalidIterableColumns || [];
		const defaultableSet = new Set(Object.keys(templateState.analysis.defaultFallbacks ?? {}));

		const missingVariables = requiredVariables.filter(
			(variable) => !availableColumns.includes(variable)
		);
		const emptyVariables = requiredVariables
			.filter((variable) => availableColumns.includes(variable))
			.filter((variable) => !columnsWithData.includes(variable))
			.filter((variable) => !defaultableSet.has(variable))
			.filter((variable) => !conditionalVariableSet.value.has(variable));
		const invalidIterableVariables = templateState.analysis.iterableVariables
			.filter((variable) => availableColumns.includes(variable))
			.filter((variable) => invalidIterableColumns.includes(variable));

		return {
			isValid:
				missingVariables.length === 0
				&& emptyVariables.length === 0
				&& invalidIterableVariables.length === 0,
			missingVariables,
			emptyVariables,
			invalidIterableVariables
		};
	});

	const defaultFallbackEntries = computed(() => {
		const map = templateState.analysis?.defaultFallbacks;
		if (!map) return [];
		return Object.entries(map);
	});

	const fallbackProvidersMap = computed(() => {
		const raw = templateState.analysis?.defaultFallbacks;
		const result = new Map<string, string[]>();
		if (!raw) return result;
		Object.entries(raw).forEach(([variable, fallback]) => {
			if (!fallback) return;
			const list = result.get(fallback) ?? [];
			list.push(variable);
			result.set(fallback, list);
		});
		return result;
	});

	const iterableVariableSet = computed(() => {
		return new Set(templateState.analysis?.iterableVariables ?? []);
	});

	const templateHintOptions = computed(() => ({
		iterableFields: templateState.analysis?.iterableFields ?? {},
		sampleValues: templateState.analysis?.sampleValues ?? {},
		defaultFallbacks: templateState.analysis?.defaultFallbacks ?? {},
		fallbackProviders: fallbackProvidersMap.value
	}));

	const variableInsights = computed<TemplateVariableInsight[]>(() => {
		if (!templateState.analysis) return [];
		return templateState.analysis.variables.map((variable) => {
			const tags = {
				loop: iterableVariableSet.value.has(variable),
				conditional: conditionalVariableSet.value.has(variable),
				defaultable: Boolean(templateState.analysis?.defaultFallbacks?.[variable]),
				formatting: isFormattingOnly(variable)
			};
			const formattingDescriptions = getFormattingDescriptions(variable);
			return {
				name: variable,
				sample: buildSampleValue(variable, tags, templateHintOptions.value),
				tags,
				formattingDescriptions,
				defaultFallback: templateState.analysis?.defaultFallbacks?.[variable],
				conditionalValues: templateState.analysis?.sampleValues?.[variable] ?? []
			};
		});
	});

	const variableInsightsMap = computed(() => {
		return new Map(variableInsights.value.map((insight) => [insight.name, insight]));
	});

	const canGenerateConfigs = computed(() => {
		return templateState.analysis
			&& excelState.preview
			&& columnValidationStatus.value.isValid;
	});

	const displayConfigs = computed(() => {
		if (showAllConfigs.value) return generatedConfigs.value;
		return generatedConfigs.value.slice(0, 5);
	});

	const analyzeTemplate = async () => {
		if (!templateState.filePath) return;

		templateState.isLoading = true;
		try {
			const analysis = await useTauriCoreInvoke<TeraTemplateAnalysis>("analyze_tera_template", {
				request: {
					filePath: templateState.filePath
				}
			});

			templateState.analysis = analysis;
			toast.add({
				title: "模板分析成功",
				description: `检测到 ${analysis.variableCount} 个变量`,
				color: "success"
			});
		} catch (error) {
			templateState.analysis = null;
			toast.add({
				title: "模板分析失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			templateState.isLoading = false;
		}
	};

	const handleSelectTemplate = async () => {
		try {
			const { open } = await import("@tauri-apps/plugin-dialog");
			const selected = await open({
				multiple: false,
				filters: [
					{ name: "模板文件", extensions: ["txt", "template", "tera"] },
					{ name: "所有文件", extensions: ["*"] }
				]
			});
			if (!selected) return;
			const path = Array.isArray(selected) ? selected[0] : selected;
			if (!path) return;

			templateState.filePath = path;
			await analyzeTemplate();
		} catch (error) {
			toast.add({
				title: "选择模板文件失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	};

	const handleReanalyzeTemplate = () => {
		if (templateState.filePath) {
			analyzeTemplate();
		}
	};

	const handleExportVariableTemplate = async () => {
		if (!templateState.analysis) return;

		try {
			const { save } = await import("@tauri-apps/plugin-dialog");
			const path = await save({
				defaultPath: "模板变量.xlsx",
				filters: [{ name: "Excel", extensions: ["xlsx"] }]
			});
			if (!path) return;

			await useTauriCoreInvoke("export_tera_variable_template", {
				request: {
					path,
					variables: templateState.analysis.variables,
					iterableVariables: templateState.analysis.iterableVariables,
					iterableFields: templateState.analysis.iterableFields || {},
					sampleValues: templateState.analysis.sampleValues || {},
					defaultFallbacks: templateState.analysis.defaultFallbacks || {},
					filterUsage: templateState.analysis.filterUsage || {}
				}
			});

			toast.add({
				title: "变量模板导出成功",
				description: `模板已保存到 ${path}（首行为变量名，第二行为示例，可按需修改）`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "导出失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	};

	const handleSelectExcel = async () => {
		try {
			const { open } = await import("@tauri-apps/plugin-dialog");
			const selected = await open({
				multiple: false,
				filters: [
					{ name: "Excel", extensions: ["xlsx", "xls"] }
				]
			});
			if (!selected) return;
			const path = Array.isArray(selected) ? selected[0] : selected;
			if (!path) return;

			excelState.filePath = path;
			await previewExcel();
		} catch (error) {
			toast.add({
				title: "选择Excel文件失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	};

	const handleRepreviewExcel = () => {
		if (!excelState.filePath) return;
		const sheetToUse = excelState.selectedSheet || undefined;
		previewExcel(sheetToUse);
	};

	const previewExcel = async (sheetName?: string) => {
		if (!excelState.filePath || !templateState.analysis) return;

		excelState.isLoading = true;
		try {
			const preview = await useTauriCoreInvoke<TemplateExcelPreview>("preview_template_excel", {
				request: {
					filePath: excelState.filePath,
					sheetName: sheetName || excelState.selectedSheet || undefined,
					expectedVariables: templateState.analysis.variables,
					iterableVariables: templateState.analysis.iterableVariables
				}
			});

			excelState.preview = preview;
			excelState.selectedSheet = preview.selectedSheet;

			toast.add({
				title: "Excel 预览成功",
				description: `加载工作表 "${preview.selectedSheet}"，共 ${preview.totalRows} 行数据`,
				color: "success"
			});
		} catch (error) {
			excelState.preview = null;
			toast.add({
				title: "Excel 预览失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			excelState.isLoading = false;
		}
	};

	const onSheetChange = async () => {
		if (excelState.selectedSheet) {
			await previewExcel(excelState.selectedSheet);
		}
	};

	const clearExcelData = () => {
		excelState.filePath = "";
		excelState.preview = null;
		excelState.selectedSheet = "";
		if (templateState.analysis?.variables.length) {
			labelField.value = templateState.analysis.variables[0];
		}
		generationErrors.value = [];
		generatedConfigs.value = [];
		expandedConfigs.value = {};
		showAllConfigs.value = false;
	};

	const generateConfigs = async () => {
		if (!templateState.analysis || !excelState.preview) return;

		generationLoading.value = true;
		generationErrors.value = [];

		try {
			const configs = await useTauriCoreInvoke<GenericGeneratedConfig[]>("generate_template_configs", {
				request: {
					templatePath: templateState.filePath,
					excelPath: excelState.filePath,
					sheetName: excelState.selectedSheet || undefined,
					expectedVariables: templateState.analysis.variables,
					labelField: labelField.value || undefined,
					iterableVariables: templateState.analysis.iterableVariables
				}
			});

			generatedConfigs.value = configs;
			expandedConfigs.value = {};

			toast.add({
				title: "配置生成成功",
				description: `共生成 ${configs.length} 个配置`,
				color: "success"
			});
		} catch (error) {
			generationErrors.value = [extractErrorMessage(error)];
			toast.add({
				title: "配置生成失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			generationLoading.value = false;
		}
	};

	const regenerateConfigs = async () => {
		await generateConfigs();
	};

	const toggleConfigExpansion = (index: number) => {
		expandedConfigs.value[index] = !expandedConfigs.value[index];
	};

	const copyConfig = async (config: string) => {
		try {
			await navigator.clipboard.writeText(config);
			toast.add({
				title: "复制成功",
				description: "配置已复制到剪贴板",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "复制失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	};

	const exportToExcel = async () => {
		try {
			const { save } = await import("@tauri-apps/plugin-dialog");
			const path = await save({
				defaultPath: "生成的配置.xlsx",
				filters: [{ name: "Excel", extensions: ["xlsx"] }]
			});
			if (!path) return;

			await useTauriCoreInvoke("export_template_configs", {
				path,
				configs: generatedConfigs.value
			});

			toast.add({
				title: "导出成功",
				description: `配置已导出到 ${path}`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "导出失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	};

	const getLoopCountText = () => {
		if (!templateState.analysis) return "未使用";
		if (templateState.analysis.hasLoops) {
			return `${templateState.analysis.loopCount} 个`;
		}
		return "未使用";
	};

	const getConditionalCountText = () => {
		if (!templateState.analysis) return "未使用";
		if (templateState.analysis.hasConditionals) {
			return `${templateState.analysis.conditionalCount} 个`;
		}
		return "未使用";
	};

	const getDefaultFallbackText = () => {
		const fallbackCount = defaultFallbackEntries.value.length;
		return fallbackCount > 0 ? `${fallbackCount} 列` : "未使用";
	};

	const getDefaultFallback = (variable: string) => {
		return templateState.analysis?.defaultFallbacks?.[variable];
	};

	const isFormattingOnly = (variable: string) => {
		const filters = templateState.analysis?.filterUsage?.[variable];
		if (!filters || !filters.length) return false;
		return filters.every((filter) => FORMATTING_FILTERS.includes(filter as typeof FORMATTING_FILTERS[number]));
	};

	const getFormattingDescriptions = (variable: string) => {
		const filters = templateState.analysis?.filterUsage?.[variable];
		if (!filters || !filters.length) return [];
		const unique = Array.from(new Set(filters));
		return unique
			.map((filter) => FORMATTING_DESCRIPTIONS[filter] || `自动应用 ${filter} 过滤器`)
			.filter(Boolean);
	};

	return {
		templateState,
		excelState,
		labelField,
		generationLoading,
		generationErrors,
		generatedConfigs,
		expandedConfigs,
		showAllConfigs,
		excelPreviewColumns,
		excelPreviewRows,
		columnValidationStatus,
		defaultFallbackEntries,
		fallbackProvidersMap,
		variableInsights,
		variableInsightsMap,
		canGenerateConfigs,
		displayConfigs,
		handleSelectTemplate,
		handleReanalyzeTemplate,
		handleExportVariableTemplate,
		handleSelectExcel,
		handleRepreviewExcel,
		clearExcelData,
		generateConfigs,
		regenerateConfigs,
		exportToExcel,
		toggleConfigExpansion,
		copyConfig,
		onSheetChange,
		getLoopCountText,
		getConditionalCountText,
		getDefaultFallbackText,
		getDefaultFallback,
		formatExampleList
	};
}
