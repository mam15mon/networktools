<template>
	<LayoutTile
		title="模板批量配置生成"
		description="基于 Tera 模板引擎，从 Excel 数据批量生成配置文件，支持复杂变量映射和条件渲染。"
	>
		<div class="space-y-8">
			<!-- 模板文件选择 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-file-text" class="size-5" />
						<h3 class="text-lg font-semibold">
							Tera 模板文件
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<div class="flex flex-wrap items-center gap-3">
						<UButton
							:loading="templateState.isLoading"
							icon="i-lucide-folder-open"
							@click="handleSelectTemplate"
						>
							选择模板文件
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-refresh-cw"
							:disabled="!templateState.filePath || templateState.isLoading"
							@click="handleReanalyzeTemplate"
						>
							重新解析
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-download"
							:disabled="!templateState.analysis"
							@click="handleExportVariableTemplate"
						>
							导出变量模板
						</UButton>
					</div>

					<div v-if="templateState.filePath" class="p-3 bg-(--ui-bg-muted) rounded-md">
						<p class="text-sm text-(--ui-text-muted) break-all">
							<span class="font-medium">当前文件：</span>{{ templateState.filePath }}
						</p>
					</div>

					<div v-if="templateState.analysis" class="space-y-4">
						<div class="grid gap-4 md:grid-cols-4">
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-variable" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">检测变量</span>
								</div>
								<p class="text-lg font-semibold">{{ templateState.analysis.variableCount }}</p>
							</div>
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-git-branch" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">循环控制</span>
								</div>
								<p class="text-lg font-semibold">{{ getLoopCountText() }}</p>
							</div>
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-git-merge" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">条件渲染</span>
								</div>
								<p class="text-lg font-semibold">{{ getConditionalCountText() }}</p>
							</div>
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-rotate-ccw" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">默认回退</span>
								</div>
								<p class="text-lg font-semibold">{{ getDefaultFallbackText() }}</p>
								<p class="text-xs text-(--ui-text-muted) mt-1">
									使用 default 过滤器，可留空
								</p>
							</div>
													</div>

						<div v-if="templateState.analysis.variables.length" class="space-y-3">
							<div class="flex items-center justify-between">
								<h4 class="text-base font-semibold">
									模板变量列表
								</h4>
								<p class="text-xs text-(--ui-text-muted)">
									点击变量可设为标识字段
								</p>
							</div>
							<div class="flex flex-wrap gap-2">
								<UBadge
									v-for="variable in templateState.analysis.variables"
									:key="variable"
									:variant="labelField === variable ? 'solid' : 'outline'"
									:color="labelField === variable ? 'primary' : 'default'"
									size="sm"
									class="cursor-pointer transition-all hover:scale-105"
									@click="setLabelField(variable)"
								>
									<div class="flex items-center gap-1">
										{{ variable }}
										<UTooltip
											v-if="hasLoopUsage(variable)"
											text="在 {% for %} 中使用，Excel 中需填 JSON 数组/对象"
											:popper="{ arrow: true }"
										>
											<Icon
												name="i-lucide-repeat-2"
												class="size-3 text-blue-500"
											/>
										</UTooltip>
										<UTooltip
											v-if="hasConditionalUsage(variable)"
											text="参与条件渲染，请按模板示例填写"
											:popper="{ arrow: true }"
										>
											<Icon
												name="i-lucide-git-branch"
												class="size-3 text-emerald-500"
											/>
										</UTooltip>
										<UTooltip
											v-if="getDefaultFallback(variable)"
											text="支持 default 过滤器，可留空"
											:popper="{ arrow: true }"
										>
											<Icon
												name="i-lucide-circle-dot"
												class="size-3 text-amber-500"
											/>
										</UTooltip>
										<Icon
											v-if="labelField === variable"
											name="i-lucide-check"
											class="size-3"
										/>
									</div>
								</UBadge>
							</div>
						</div>

						<div v-if="defaultFallbackEntries.length" class="space-y-2">
							<h4 class="text-base font-semibold">
								支持 default 回退的变量
							</h4>
							<p class="text-xs text-(--ui-text-muted)">
								这些列留空时会自动使用默认值，请确保 Excel 中的默认来源列存在。
							</p>
							<div class="flex flex-wrap gap-2">
								<UBadge
									v-for="([variable, fallback], index) in defaultFallbackEntries"
									:key="`default-${variable}-${index}`"
									variant="outline"
									size="sm"
								>
									{{ variable }} ← {{ fallback }}
								</UBadge>
							</div>
						</div>

						<div v-if="loopVariables.length" class="space-y-2">
							<h4 class="text-base font-semibold">
								循环使用的变量
							</h4>
							<p class="text-xs text-(--ui-text-muted)">
								这些变量会在 {% for %} 中迭代，Excel 中需填写 JSON 数组或对象结构。
							</p>
							<div class="flex flex-wrap gap-2">
								<UBadge
									v-for="variable in loopVariables"
									:key="`loop-${variable}`"
									variant="outline"
									size="sm"
								>
									{{ variable }}
								</UBadge>
							</div>
						</div>

						<div v-if="conditionalEntries.length" class="space-y-2">
							<h4 class="text-base font-semibold">
								参与条件渲染的变量
							</h4>
							<p class="text-xs text-(--ui-text-muted)">
								根据模板中的 if/elif 比较，这些变量通常取以下值，请按需填写。
							</p>
							<div class="flex flex-col gap-1">
								<div
									v-for="([variable, values], index) in conditionalEntries"
									:key="`conditional-${variable}-${index}`"
									class="text-sm text-(--ui-text)"
								>
									<span class="font-medium">{{ variable }}</span>
									<span class="text-(--ui-text-muted)"> → {{ formatExampleList(values) }}</span>
								</div>
							</div>
						</div>
					</div>
				</div>
			</UCard>

			<!-- Excel 数据文件 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-file-spreadsheet" class="size-5" />
						<h3 class="text-lg font-semibold">
							Excel 数据文件
						</h3>
					</div>
				</template>
				<div class="space-y-6">
					<div class="flex flex-wrap items-center gap-3">
						<UButton
							:loading="excelState.isLoading"
							icon="i-lucide-folder-open"
							@click="handleSelectExcel"
						>
							选择 Excel 文件
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-refresh-cw"
							:disabled="!excelState.filePath || excelState.isLoading"
							@click="handleRepreviewExcel"
						>
							重新解析
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-eraser"
							:disabled="!excelState.preview"
							@click="clearExcelData"
						>
							清除数据
						</UButton>
					</div>

					<div v-if="excelState.filePath" class="p-3 bg-(--ui-bg-muted) rounded-md">
						<p class="text-sm text-(--ui-text-muted) break-all">
							<span class="font-medium">当前文件：</span>{{ excelState.filePath }}
						</p>
					</div>

					<div v-if="excelState.preview" class="space-y-4">
						<div class="grid gap-4 md:grid-cols-[220px_1fr] items-center">
							<label class="text-sm font-medium text-(--ui-text-muted)">
								工作表
							</label>
							<USelect
								v-model="excelState.selectedSheet"
								:options="excelState.preview.sheetNames"
								placeholder="选择工作表"
								class="w-full"
								@change="onSheetChange"
							/>
						</div>

						<div v-if="excelState.preview.columns.length" class="space-y-3">
								<div class="flex items-center justify-between">
									<h4 class="text-base font-semibold">
										列映射验证
									</h4>
									<div class="flex items-center gap-2">
										<div v-if="columnValidationStatus.isValid" class="flex items-center gap-1 text-xs text-green-600">
											<Icon name="i-lucide-check-circle" class="size-3" />
											<span>验证通过</span>
										</div>
										<div v-else class="flex items-center gap-1 text-xs text-red-600">
											<Icon name="i-lucide-alert-circle" class="size-3" />
											<span>
												<template v-if="columnValidationStatus.missingVariables.length">
													缺少必需列
												</template>
												<template v-else>
													必需列缺少有效数据
												</template>
											</span>
										</div>
									</div>
								</div>

								<div v-if="columnValidationStatus.missingVariables.length" class="space-y-2">
									<p class="text-sm text-red-600">
										缺少以下变量对应的列：
									</p>
									<div class="flex flex-wrap gap-2">
										<UBadge
											v-for="variable in columnValidationStatus.missingVariables"
											:key="variable"
											variant="error"
											size="sm"
										>
											{{ variable }}
										</UBadge>
									</div>
								</div>

								<div v-if="columnValidationStatus.emptyVariables.length" class="space-y-2">
									<p class="text-sm text-amber-600">
										以下列存在但没有可用数据，无法生成配置：
									</p>
									<div class="flex flex-wrap gap-2">
										<UBadge
											v-for="variable in columnValidationStatus.emptyVariables"
											:key="`empty-${variable}`"
											variant="outline"
											size="sm"
										>
											{{ variable }}
										</UBadge>
									</div>
									<p class="text-xs text-(--ui-text-muted)">
										请在 Excel 中至少为这些列提供一条非空数据后重新解析。
									</p>
								</div>

								<div v-if="columnValidationStatus.invalidIterableVariables.length" class="space-y-2">
									<p class="text-sm text-amber-600">
										以下列必须填写“由 JSON 对象组成的数组”或“单个 JSON 对象”（示例：
										<code class="px-1 py-0.5 rounded bg-(--ui-bg-muted)">
											[{"id":1,"name":"Core","ip":"10.0.0.1","mask":"255.255.255.0"}]
										</code>
										），且数组元素不得再是数组，请检查填充内容：
									</p>
									<div class="flex flex-wrap gap-2">
										<UBadge
											v-for="variable in columnValidationStatus.invalidIterableVariables"
											:key="`invalid-${variable}`"
											variant="outline"
											size="sm"
										>
											{{ variable }}
										</UBadge>
									</div>
									<p class="text-xs text-(--ui-text-muted)">
										模板中通过 <code>{% for %}</code> 使用这些变量时，Excel 列必须提供合法 JSON 结构，否则无法生成配置。
									</p>
								</div>

							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<h4 class="text-base font-semibold">
										数据预览（{{ excelState.preview.totalRows }} 行，展示前 {{ excelPreviewRows.length }} 行）
									</h4>
								</div>
								<UTable
									:columns="excelPreviewColumns"
									:data="excelPreviewRows"
									class="w-full"
								/>
							</div>
						</div>
					</div>
				</div>
			</UCard>

			
			<!-- 配置生成 -->
			<UCard v-if="canGenerateConfigs" class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-cpu" class="size-5" />
						<h3 class="text-lg font-semibold">
							配置生成
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<div class="flex flex-wrap items-center gap-3">
						<UButton
							:loading="generationLoading"
							icon="i-lucide-play"
							@click="generateConfigs"
						>
							生成配置
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-refresh-cw"
							:disabled="!generatedConfigs.length"
							@click="regenerateConfigs"
						>
							重新生成
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-file-spreadsheet"
							:disabled="!generatedConfigs.length"
							@click="exportToExcel"
						>
							导出Excel
						</UButton>
					</div>

					<div v-if="generationErrors.length" class="space-y-2">
						<UAlert variant="error" icon="i-lucide-alert-triangle" title="生成错误">
							<ul class="list-disc pl-5 space-y-1">
								<li v-for="(error, index) in generationErrors" :key="`error-${index}`">
									{{ error }}
								</li>
							</ul>
						</UAlert>
					</div>

					<div v-if="generatedConfigs.length" class="space-y-4">
						<div class="flex items-center justify-between">
							<h4 class="text-base font-semibold">
								生成结果（{{ generatedConfigs.length }} 个配置）
							</h4>
							<div class="flex items-center gap-2">
								<UButton
									variant="outline"
									size="sm"
									icon="i-lucide-chevron-down"
									@click="showAllConfigs = !showAllConfigs"
								>
									{{ showAllConfigs ? '收起' : '展开' }}全部
								</UButton>
							</div>
						</div>

						<div class="space-y-3">
							<div
								v-for="(config, index) in displayConfigs"
								:key="config.rowIndex"
								class="border border-(--ui-border) rounded-lg overflow-hidden"
							>
								<div
									class="p-3 bg-(--ui-bg-muted) cursor-pointer flex items-center justify-between"
									@click="toggleConfigExpansion(index)"
								>
									<div class="flex items-center gap-3">
										<Icon
											:name="expandedConfigs[index] ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
											class="size-4 transition-transform"
										/>
										<span class="font-medium">{{ config.label }}</span>
										<UBadge variant="outline" size="sm">
											第 {{ config.rowIndex }} 行
										</UBadge>
									</div>
									<div class="flex items-center gap-2">
										<UButton
											variant="ghost"
											size="xs"
											icon="i-lucide-copy"
											@click.stop="copyConfig(config.config)"
										/>
									</div>
								</div>
								<div v-if="expandedConfigs[index]" class="p-4 border-t border-(--ui-border)">
									<pre class="text-sm font-mono whitespace-pre-wrap bg-(--ui-bg) p-3 rounded border border-(--ui-border)">{{ config.config }}</pre>
								</div>
							</div>
						</div>

						<div v-if="generatedConfigs.length > displayConfigs.length" class="text-center">
							<UButton
								variant="outline"
								@click="showAllConfigs = true"
							>
								显示剩余 {{ generatedConfigs.length - displayConfigs.length }} 个配置
							</UButton>
						</div>
					</div>
				</div>
			</UCard>

					</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import type {
		TeraTemplateAnalysis,
		TemplateExcelPreview,
		GenericGeneratedConfig
	} from "~/types/template-batch";
	import { computed, reactive, ref } from "vue";
	import { extractErrorMessage } from "~/utils/error";

	definePageMeta({
		name: "模板批量生成",
		icon: "lucide:file-text",
		description: "基于模板批量生成配置文件",
		category: "other"
	});

	const toast = useToast();

	// 模板状态
	const templateState = reactive({
		filePath: "",
		analysis: null as TeraTemplateAnalysis | null,
		isLoading: false
	});

	// Excel 状态
	const excelState = reactive({
		filePath: "",
		preview: null as TemplateExcelPreview | null,
		selectedSheet: "",
		isLoading: false
	});

	// 其他状态
	const labelField = ref("");

// 当模板分析完成时，自动选择第一个变量作为标识字段
watch(
	() => templateState.analysis?.variables,
	(variables) => {
		if (variables && variables.length > 0 && !labelField.value) {
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

	// 计算属性
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

	const columnValidationStatus = computed(() => {
		if (!templateState.analysis || !excelState.preview) {
			return { isValid: false, missingVariables: [], emptyVariables: [], invalidIterableVariables: [] };
		}

		const requiredVariables = templateState.analysis.variables;
		const availableColumns = excelState.preview.columns;
		const columnsWithData = excelState.preview.columnsWithData || [];
		const invalidIterableColumns = excelState.preview.invalidIterableColumns || [];

		const missingVariables = requiredVariables.filter(
			variable => !availableColumns.includes(variable)
		);
		const emptyVariables = requiredVariables
			.filter(variable => availableColumns.includes(variable))
			.filter(variable => !columnsWithData.includes(variable));
		const invalidIterableVariables = templateState.analysis.iterableVariables
			.filter(variable => availableColumns.includes(variable))
			.filter(variable => invalidIterableColumns.includes(variable));

		return {
			isValid:
				missingVariables.length === 0 &&
				emptyVariables.length === 0 &&
				invalidIterableVariables.length === 0,
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

	const iterableVariableSet = computed(() => {
		return new Set(templateState.analysis?.iterableVariables ?? []);
	});

	const conditionalVariableSet = computed(() => {
		const map = templateState.analysis?.sampleValues;
		if (!map) return new Set<string>();
		const entries = Object.entries(map).filter(([, values]) => values && values.length > 0);
		return new Set(entries.map(([key]) => key));
	});

	const loopVariables = computed(() => {
		return Array.from(iterableVariableSet.value);
	});

	const conditionalEntries = computed(() => {
		const map = templateState.analysis?.sampleValues;
		if (!map) return [];
		return Object.entries(map).filter(([, values]) => values && values.length > 0);
	});

	
	const canGenerateConfigs = computed(() => {
		return templateState.analysis &&
			   excelState.preview &&
			   columnValidationStatus.value.isValid;
	});

	const displayConfigs = computed(() => {
		if (showAllConfigs.value) return generatedConfigs.value;
		return generatedConfigs.value.slice(0, 5);
	});

	
	// 方法
	function handleReanalyzeTemplate() {
		if (templateState.filePath) {
			analyzeTemplate();
		}
	}

	function handleRepreviewExcel() {
		if (excelState.filePath) {
			// 重新解析时，使用当前选择的工作表，如果没有则让系统自动选择
			const sheetToUse = excelState.selectedSheet || undefined;
			previewExcel(sheetToUse);
		}
	}

	function getLoopCountText(): string {
		if (!templateState.analysis) return "未使用";
		if (templateState.analysis.hasLoops) {
			return `${templateState.analysis.loopCount} 个`;
		}
		return "未使用";
	}

	function getConditionalCountText(): string {
		if (!templateState.analysis) return "未使用";
		if (templateState.analysis.hasConditionals) {
			return `${templateState.analysis.conditionalCount} 个`;
		}
		return "未使用";
	}

	function getDefaultFallbackText(): string {
		const fallbackCount = defaultFallbackEntries.value.length;
		return fallbackCount > 0 ? `${fallbackCount} 列` : "未使用";
	}

	function getDefaultFallback(variable: string): string | undefined {
		return templateState.analysis?.defaultFallbacks?.[variable];
	}

	function hasLoopUsage(variable: string): boolean {
		return iterableVariableSet.value.has(variable);
	}

	function hasConditionalUsage(variable: string): boolean {
		return conditionalVariableSet.value.has(variable);
	}

	function formatExampleList(values: string[]): string {
		if (!values || !values.length) return "示例缺失";
		const preview = values.slice(0, 3).join(" / ");
		return values.length > 3 ? `${preview} / ...` : preview;
	}

	function setLabelField(variable: string) {
		if (labelField.value === variable) {
			// 如果点击的是已选中的变量，则取消选择
			labelField.value = "";
		} else {
			// 设置新的标识字段
			labelField.value = variable;
		}
	}

	async function handleSelectTemplate() {
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
	}

	async function analyzeTemplate() {
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
	}

	async function handleExportVariableTemplate() {
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
	}

	async function handleSelectExcel() {
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
	}

	async function previewExcel(sheetName?: string) {
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
	}

	async function onSheetChange() {
		if (excelState.selectedSheet) {
			await previewExcel(excelState.selectedSheet);
		}
	}

	function clearExcelData() {
		excelState.filePath = "";
		excelState.preview = null;
		excelState.selectedSheet = "";
		// 恢复到第一个变量作为标识字段
		if (templateState.analysis?.variables.length) {
			labelField.value = templateState.analysis.variables[0];
		}
		generationErrors.value = [];
		generatedConfigs.value = [];
		expandedConfigs.value = {};
		showAllConfigs.value = false;
	}

	async function generateConfigs() {
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
	}

	async function regenerateConfigs() {
		await generateConfigs();
	}

	function toggleConfigExpansion(index: number) {
		expandedConfigs.value[index] = !expandedConfigs.value[index];
	}

	async function copyConfig(config: string) {
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
	}

	
	async function exportToExcel() {
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
	}
</script>
