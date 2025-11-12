<template>
	<LayoutTile
		title="模板批量配置生成"
		description="基于 Tera 模板引擎，从 Excel 数据批量生成配置文件，支持复杂变量映射和条件渲染。"
	>
		<div class="space-y-8 w-full max-w-screen-2xl mx-auto">
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
						<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
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
						<div class="grid gap-4 md:grid-cols-[minmax(220px,260px)_1fr] items-center">
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
					以下列必须填写“由 JSON 对象组成的数组”、“单个 JSON 对象”或“简单值数组”（示例：
					<code class="px-1 py-0.5 rounded bg-(--ui-bg-muted)">
						[{"id":1,"name":"Core"}] / {"id":1} / ["vlan10","vlan20"]
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

			<GenerationActions
				v-if="canGenerateConfigs"
				:generation-loading="generationLoading"
				:generation-errors="generationErrors"
				:generated-configs="generatedConfigs"
				:display-configs="displayConfigs"
				:expanded-configs="expandedConfigs"
				:show-all-configs="showAllConfigs"
				@generate="generateConfigs"
				@regenerate="regenerateConfigs"
				@export="exportToExcel"
				@update:show-all-configs="showAllConfigs = $event"
				@toggle-config="toggleConfigExpansion"
				@copy-config="copyConfig"
			/>
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
					<div v-if="shouldDisplaySample(activeVariableInsight)">
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
	import { computed, reactive } from "vue";
	import GenerationActions from "~/components/template-batch/GenerationActions.vue";
	import { useTemplateBatch, type TemplateVariableInsight } from "~/composables/useTemplateBatch";
	import { shouldShowSample } from "~/utils/templateHints";

	definePageMeta({
		name: "模板批量生成",
		icon: "lucide:file-text",
		description: "基于模板批量生成配置文件",
		category: "other"
	});

	const {
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
		formatExampleList
	} = useTemplateBatch();

	const variableDrawerDescription = "查看变量的示例值与校验提示";
	const variableDrawer = reactive({
		open: false,
		variable: ""
	});

	const activeVariableInsight = computed(() => {
		if (!variableDrawer.variable) return null;
		return variableInsightsMap.value.get(variableDrawer.variable) ?? null;
	});

	const shouldDisplaySample = (insight: TemplateVariableInsight | null) => {
		if (!insight) return false;
		return shouldShowSample(insight.name, insight.sample);
	};

	const setLabelField = (variable: string) => {
		labelField.value = labelField.value === variable ? "" : variable;
	};

	const openVariableDrawer = (variable: string) => {
		variableDrawer.variable = variable;
		variableDrawer.open = true;
	};

	const closeVariableDrawer = () => {
		variableDrawer.open = false;
	};
</script>
