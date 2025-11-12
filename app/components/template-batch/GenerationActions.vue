<template>
	<UCard class="bg-(--ui-bg)">
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
					@click="$emit('generate')"
				>
					生成配置
				</UButton>
				<UButton
					variant="outline"
					icon="i-lucide-refresh-cw"
					:disabled="!generatedConfigs.length"
					@click="$emit('regenerate')"
				>
					重新生成
				</UButton>
				<UButton
					variant="outline"
					icon="i-lucide-file-spreadsheet"
					:disabled="!generatedConfigs.length"
					@click="$emit('export')"
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
							@click="$emit('update:showAllConfigs', !showAllConfigs)"
						>
							{{ showAllConfigs ? "收起" : "展开" }}全部
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
							@click="$emit('toggle-config', index)"
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
									@click.stop="$emit('copy-config', config.config)"
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
						@click="$emit('update:showAllConfigs', true)"
					>
						显示剩余 {{ generatedConfigs.length - displayConfigs.length }} 个配置
					</UButton>
				</div>
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { GenericGeneratedConfig } from "~/types/template-batch";

	interface Props {
		generationLoading: boolean
		generationErrors: string[]
		generatedConfigs: GenericGeneratedConfig[]
		displayConfigs: GenericGeneratedConfig[]
		expandedConfigs: Record<number, boolean>
		showAllConfigs: boolean
	}

	defineProps<Props>();

	defineEmits<{
		(e: "generate"): void
		(e: "regenerate"): void
		(e: "export"): void
		(e: "update:showAllConfigs", value: boolean): void
		(e: "toggle-config", value: number): void
		(e: "copy-config", value: string): void
	}>();
</script>
