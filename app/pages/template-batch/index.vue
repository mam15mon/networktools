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
						<TemplateBatchTeraTutorialDrawer />
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
								<p class="text-lg font-semibold">
									{{ templateState.analysis.variableCount }}
								</p>
							</div>
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-git-branch" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">循环控制</span>
								</div>
								<p class="text-lg font-semibold">
									{{ getLoopCountText() }}
								</p>
							</div>
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-git-merge" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">条件渲染</span>
								</div>
								<p class="text-lg font-semibold">
									{{ getConditionalCountText() }}
								</p>
							</div>
							<div class="p-3 bg-(--ui-bg-muted) rounded-lg">
								<div class="flex items-center gap-2 mb-1">
									<Icon name="i-lucide-rotate-ccw" class="size-4 text-(--ui-text-muted)" />
									<span class="text-xs font-medium text-(--ui-text-muted)">默认回退</span>
								</div>
								<p class="text-lg font-semibold">
									{{ getDefaultFallbackText() }}
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
										<Icon
											v-if="labelField === variable"
											name="i-lucide-check"
											class="size-3"
										/>
									</div>
								</UBadge>
							</div>
						</div>

						<div v-if="variableInsights.length" class="space-y-2">
							<h4 class="text-base font-semibold">
								变量示例预览
							</h4>
							<div class="overflow-x-auto border border-(--ui-border) rounded-lg">
								<table class="min-w-full text-sm">
									<thead class="bg-(--ui-bg-muted)">
										<tr>
											<th class="px-3 py-2 text-left font-medium text-(--ui-text-muted)">
												变量
											</th>
											<th class="px-3 py-2 text-left font-medium text-(--ui-text-muted)">
												示例值
											</th>
										</tr>
									</thead>
									<tbody>
										<tr
											v-for="insight in variableInsights"
											:key="`preview-${insight.name}`"
											class="border-t border-(--ui-border) cursor-pointer hover:bg-(--ui-bg-muted)"
											@click="openVariableDrawer(insight.name)"
										>
											<td class="px-3 py-2 font-medium text-(--ui-text)">
												{{ insight.name }}
											</td>
											<td class="px-3 py-2 font-mono text-xs text-(--ui-text-muted)">
												{{ insight.sample }}
											</td>
										</tr>
									</tbody>
								</table>
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
		<UDrawer
			v-model:open="variableDrawer.open"
			direction="right"
			title="变量详情"
			:description="variableDrawerDescription"
		>
			<template #body>
				<div v-if="activeVariableInsight" class="space-y-4 p-4">
					<div>
						<h3 class="text-lg font-semibold">
							{{ activeVariableInsight.name }}
						</h3>
						<p class="text-xs text-(--ui-text-muted)">
							变量详情
						</p>
					</div>
					<div v-if="shouldShowSample(activeVariableInsight)">
						<p class="text-sm text-(--ui-text-muted)">
							模板示例值：
						</p>
						<p class="font-mono text-xs bg-(--ui-bg-muted) p-2 rounded border border-(--ui-border)">
							{{ activeVariableInsight.sample }}
						</p>
					</div>

					<div>
						<h4 class="text-sm font-semibold">
							字段特性
						</h4>
						<div class="flex flex-wrap gap-2 mt-2">
							<UBadge v-if="activeVariableInsight.tags.loop" variant="subtle" color="blue">
								循环
							</UBadge>
							<UBadge v-if="activeVariableInsight.tags.conditional" variant="subtle" color="green">
								条件
							</UBadge>
							<UBadge v-if="activeVariableInsight.tags.defaultable" variant="subtle" color="amber">
								Default
							</UBadge>
							<UBadge v-if="activeVariableInsight.tags.formatting" variant="subtle" color="purple">
								格式化
							</UBadge>
						</div>
						<ul class="mt-3 space-y-1 text-xs text-(--ui-text-muted)">
							<li v-if="activeVariableInsight.tags.loop">
								循环变量：Excel 中必须填写 JSON 数组或对象结构，否则无法渲染 {% for %}。
							</li>
							<li v-if="activeVariableInsight.tags.conditional && activeVariableInsight.conditionalValues.length">
								条件变量：模板检测到可选值 {{ formatExampleList(activeVariableInsight.conditionalValues) }}。
							</li>
							<li v-if="activeVariableInsight.tags.defaultable && activeVariableInsight.defaultFallback">
								Default 回退：留空时将使用 {{ activeVariableInsight.defaultFallback }}。
							</li>
							<li v-if="activeVariableInsight.tags.formatting && activeVariableInsight.formattingDescriptions.length">
								格式化提示：{{ activeVariableInsight.formattingDescriptions.join(" / ") }}。
							</li>
							<li v-else-if="activeVariableInsight.tags.formatting">
								格式化变量：渲染时会自动应用过滤器，Excel 直接填写原值即可。
							</li>
							<li v-if="!activeVariableInsight.tags.loop && !activeVariableInsight.tags.conditional && !activeVariableInsight.tags.defaultable && !activeVariableInsight.tags.formatting">
								常规变量：直接在 Excel 中填写对应值。
							</li>
						</ul>
					</div>

					<div v-if="activeVariableInsight.defaultFallback">
						<h4 class="text-sm font-semibold">
							默认回退
						</h4>
						<p class="text-sm text-(--ui-text-muted)">
							留空时默认使用 <span class="font-mono">{{ activeVariableInsight.defaultFallback }}</span>
						</p>
					</div>

					<div v-if="activeVariableInsight.conditionalValues.length">
						<h4 class="text-sm font-semibold">
							条件示例
						</h4>
						<p class="text-xs text-(--ui-text-muted)">
							模板检测到以下典型取值：
						</p>
						<div class="flex flex-wrap gap-1 mt-1">
							<UBadge
								v-for="(value, index) in activeVariableInsight.conditionalValues"
								:key="`drawer-cond-${index}`"
								size="sm"
								variant="outline"
							>
								{{ value }}
							</UBadge>
						</div>
					</div>

					<div class="flex justify-end">
						<UButton variant="outline" @click="closeVariableDrawer">
							关闭
						</UButton>
					</div>
				</div>
				<div v-else class="text-sm text-(--ui-text-muted) p-4">
					未找到变量信息。
				</div>
			</template>
		</UDrawer>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import type {
		GenericGeneratedConfig,
		TemplateExcelPreview,
		TeraTemplateAnalysis
	} from "~/types/template-batch";
	import { computed, reactive, ref, watch } from "vue";
	import { extractErrorMessage } from "~/utils/error";

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
	const variableDrawerDescription = "查看变量的示例值与校验提示";
	const variableDrawer = reactive({
		open: false,
		variable: ""
	});

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
			(variable) => !availableColumns.includes(variable)
		);
		const emptyVariables = requiredVariables
			.filter((variable) => availableColumns.includes(variable))
			.filter((variable) => !columnsWithData.includes(variable));
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

	const iterableVariableSet = computed(() => {
		return new Set(templateState.analysis?.iterableVariables ?? []);
	});

	const conditionalVariableSet = computed(() => {
		const map = templateState.analysis?.sampleValues;
		if (!map) return new Set<string>();
		const entries = Object.entries(map).filter(([, values]) => values && values.length > 0);
		return new Set(entries.map(([key]) => key));
	});

	const variableInsights = computed(() => {
		if (!templateState.analysis) return [];
		return templateState.analysis.variables.map((variable) => {
			const tags = {
				loop: iterableVariableSet.value.has(variable),
				conditional: conditionalVariableSet.value.has(variable),
				defaultable: Boolean(getDefaultFallback(variable)),
				formatting: isFormattingOnly(variable)
			};
			const formattingDescriptions = getFormattingDescriptions(variable);
			return {
				name: variable,
				sample: buildSampleValue(variable, tags),
				tags,
				formattingDescriptions,
				defaultFallback: getDefaultFallback(variable),
				conditionalValues: templateState.analysis?.sampleValues?.[variable] ?? []
			};
		});
	});

	const variableInsightsMap = computed(() => {
		return new Map(variableInsights.value.map((insight) => [insight.name, insight]));
	});

	const activeVariableInsight = computed(() => {
		if (!variableDrawer.variable) return null;
		return variableInsightsMap.value.get(variableDrawer.variable) ?? null;
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

	function formatExampleList(values: string[]): string {
		if (!values || !values.length) return "示例缺失";
		const preview = values.slice(0, 3).join(" / ");
		return values.length > 3 ? `${preview} / ...` : preview;
	}

	function isFormattingOnly(variable: string): boolean {
		const filters = templateState.analysis?.filterUsage?.[variable];
		if (!filters || !filters.length) return false;
		return filters.every((filter) => FORMATTING_FILTERS.includes(filter));
	}

	function getFormattingDescriptions(variable: string): string[] {
		const filters = templateState.analysis?.filterUsage?.[variable];
		if (!filters || !filters.length) return [];
		const unique = Array.from(new Set(filters));
		return unique
			.map((filter) => FORMATTING_DESCRIPTIONS[filter] || `自动应用 ${filter} 过滤器`)
			.filter(Boolean);
	}

	function buildSampleValue(
		variable: string,
		tags: { loop: boolean, conditional: boolean, defaultable: boolean, formatting: boolean }
	): string {
		if (tags.loop) {
			return `[{"id":1,"name":"${variable}示例","ip":"10.0.0.1","mask":"255.255.255.0"}]`;
		}
		const values = templateState.analysis?.sampleValues?.[variable] ?? [];
		const defaultHint = getDefaultFallback(variable);
		if (defaultHint) {
			if (values.length) {
				return `可选：${formatExampleList(values)}；默认 ${defaultHint}`;
			}
			return `默认 ${defaultHint}，可留空`;
		}
		if (values.length) {
			return formatExampleList(values);
		}
		if (tags.formatting) {
			return `${variable} 示例值（自动格式化）`;
		}
		return `${variable} 示例值`;
	}

	function shouldShowSample(insight: {
		name: string
		sample: string
	}): boolean {
		if (!insight.sample) return false;
		const generic = `${insight.name} 示例值`;
		const formatting = `${insight.name} 示例值（自动格式化）`;
		return insight.sample !== generic && insight.sample !== formatting;
	}

	function openVariableDrawer(variable: string) {
		variableDrawer.variable = variable;
		variableDrawer.open = true;
	}

	function closeVariableDrawer() {
		variableDrawer.open = false;
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
