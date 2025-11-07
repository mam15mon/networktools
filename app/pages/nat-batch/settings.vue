<template>
	<LayoutTile
		title="弹性 IP 与运营商配置"
		description="维护弹性 IP 映射、批量导入、以及运营商 IP 数据更新。"
	>
		<div class="space-y-8">
			<div class="flex justify-end">
				<NuxtLink to="/nat-batch">
					<UButton variant="outline" size="sm" icon="i-lucide-arrow-left">
						返回生成页面
					</UButton>
				</NuxtLink>
			</div>
			<UCard class="bg-(--ui-bg)">
			<template #header>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-cloud" class="size-5" />
						<UHeading :level="3" size="md">
							弹性 IP 管理
						</UHeading>
					</div>
					<div class="flex items-center gap-2">
						<UBadge variant="soft" color="primary">
							{{ elasticMappings.length }} 条映射
						</UBadge>
						<UButton
							variant="outline"
							size="sm"
							icon="i-lucide-rotate-ccw"
							@click="loadElasticMappings"
						>
							刷新
						</UButton>
					</div>
				</div>
			</template>

			<div class="space-y-6">
				<!-- 快速添加区域 -->
				<div class="p-4 bg-(--ui-bg-muted) rounded-lg border border-(--ui-border)">
					<div class="flex items-center gap-2 mb-3">
						<Icon name="i-lucide-plus-circle" class="size-4 text-(--ui-text-muted)" />
						<h4 class="text-sm font-semibold text-(--ui-text)">快速添加映射</h4>
					</div>
					<div class="grid gap-3 md:grid-cols-3">
						<UFormField label="内部 IP" name="internalIp">
							<UInput
								v-model="newMapping.internalIp"
								placeholder="192.168.1.100"
								clearable
								size="sm"
							/>
						</UFormField>
						<UFormField label="弹性 IP" name="elasticIp">
							<UInput
								v-model="newMapping.elasticIp"
								placeholder="222.240.138.4"
								clearable
								size="sm"
							/>
						</UFormField>
						<div class="flex items-end">
							<UButton
								icon="i-lucide-plus"
								@click="addElasticMapping"
								class="w-full"
								size="sm"
							>
								添加/更新
							</UButton>
						</div>
					</div>
				</div>

				<!-- 操作工具栏 -->
				<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
					<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-4">
						<UInput
							v-model="searchKeyword"
							placeholder="搜索内部 IP 或弹性 IP"
							icon="i-lucide-search"
							class="w-full sm:w-64"
							clearable
							size="sm"
						/>
						<div class="flex items-center gap-2">
							<UButton
								variant="outline"
								color="red"
								size="sm"
								icon="i-lucide-trash-2"
								:disabled="selectedInternalIps.length === 0"
								@click="bulkDeleteMappings"
							>
								批量删除
							</UButton>
							<div v-if="selectedInternalIps.length">
								<UBadge variant="soft" color="primary">
									已选择 {{ selectedInternalIps.length }} 条
								</UBadge>
							</div>
						</div>
					</div>
				</div>

				<!-- 批量导入区域 -->
				<div class="border border-(--ui-border) rounded-lg overflow-hidden">
					<UAccordion
						:items="[{
							label: '批量导入',
							icon: 'i-lucide-upload',
							defaultOpen: false
						}]"
					>
						<template #item>
							<div class="p-4 space-y-4 bg-(--ui-bg)">
								<div class="space-y-3">
									<UFormField label="映射数据" name="bulkMapping">
										<UTextarea
											v-model="bulkMappingInput"
											:rows="4"
											placeholder="按行输入映射，例如：&#10;192.168.1.100 -> 222.240.138.4&#10;192.168.1.101 -> 222.240.138.5"
										/>
									</UFormField>
									<div class="flex flex-wrap items-center gap-3">
										<UCheckbox
											v-model="overwriteExisting"
											label="覆盖已存在的映射"
											size="sm"
										/>
										<UButton
											variant="soft"
											size="sm"
											icon="i-lucide-upload"
											:disabled="!bulkMappingInput.trim()"
											@click="bulkAddElasticMappings"
										>
											导入映射
										</UButton>
									</div>
								</div>
							</div>
						</template>
					</UAccordion>
				</div>

				<!-- 数据表格 -->
				<UTable
					:columns="elasticColumns"
					:data="filteredElasticMappings"
					:row-key="row => row.internalIp"
					class="w-full"
					empty="暂无映射，请先添加。"
				>
					<template #select-header>
						<div class="flex justify-center">
							<UCheckbox :model-value="isAllSelected" @update:model-value="toggleSelectAll" />
						</div>
					</template>
					<template #select-data="{ row }">
						<div class="flex justify-center">
							<UCheckbox
								:model-value="selectedInternalIps.includes(row.original.internalIp)"
								@update:model-value="updateSelection(row.original.internalIp, $event)"
							/>
						</div>
					</template>
					<template #actions-data="{ row }">
						<div class="flex justify-center">
							<UButton
								variant="ghost"
								size="xs"
								icon="i-lucide-trash"
								@click="removeElasticMapping(row.original.internalIp)"
							/>
						</div>
					</template>
				</UTable>
			</div>
		</UCard>

			<UCard class="bg-(--ui-bg)">
	<template #header>
		<div class="flex items-center gap-2">
			<Icon name="i-lucide-globe-2" class="size-5" />
			<UHeading :level="3" size="md">
				运营商数据更新
			</UHeading>
		</div>
	</template>
				<div class="space-y-4">
					<div class="flex flex-wrap gap-3">
						<UButton
							:loading="ispUpdateLoading"
							icon="i-lucide-cloud-download"
							@click="updateIspDatabase"
						>
							从 GitHub 更新
						</UButton>
		<div v-if="ispSummary" class="flex flex-wrap gap-2">
			<UBadge variant="soft" color="primary">
				电信 {{ ispSummary.dxCount }}
			</UBadge>
			<UBadge variant="soft" color="primary">
				联通 {{ ispSummary.ltCount }}
			</UBadge>
			<UBadge variant="soft" color="primary">
				移动 {{ ispSummary.ydCount }}
			</UBadge>
			<UBadge variant="soft" color="primary">
				其他 {{ ispSummary.otherCount }}
			</UBadge>
		</div>
					</div>
					<div v-if="ispSummary" class="text-sm text-(--ui-text-muted)">
						数据保存位置：{{ ispSummary.savedPath }}
					</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import type { BulkElasticResult, ElasticMappingEntry, IspUpdateResult } from "~/types/nat-batch";
	import { computed, onMounted, reactive, ref, watch } from "vue";
	import { extractErrorMessage } from "~/utils/error";

	definePageMeta({
		name: "弹性 IP 与运营商配置",
		icon: "lucide:cloud",
		description: "管理弹性 IP 映射和运营商数据",
		category: "firewall",
		hidden: true
	});

	const toast = useToast();

	const elasticMappings = ref<ElasticMappingEntry[]>([]);
	const newMapping = reactive({ internalIp: "", elasticIp: "" });
	const bulkMappingInput = ref("");
	const overwriteExisting = ref(false);
	const ispUpdateLoading = ref(false);
	const ispSummary = ref<IspUpdateResult | null>(null);
	const searchKeyword = ref("");
	const selectedInternalIps = ref<string[]>([]);

	const filteredElasticMappings = computed(() => {
		const keyword = searchKeyword.value.trim().toLowerCase();
		if (!keyword) {
			return elasticMappings.value;
		}
		return elasticMappings.value.filter((mapping) => {
			return (
				mapping.internalIp.toLowerCase().includes(keyword)
				|| mapping.elasticIp.toLowerCase().includes(keyword)
			);
		});
	});

	const elasticColumns = [
		{ id: "select", accessorKey: "select", header: "" },
		{ accessorKey: "internalIp", header: "内部 IP", id: "internalIp" },
		{ accessorKey: "elasticIp", header: "弹性 IP", id: "elasticIp" },
		{ id: "actions", accessorKey: "actions", header: "操作" }
	];

	const isAllSelected = computed(() => {
		const current = filteredElasticMappings.value;
		if (!current.length) return false;
		return current.every((mapping) => selectedInternalIps.value.includes(mapping.internalIp));
	});

	watch(elasticMappings, () => {
		const existing = new Set(elasticMappings.value.map((mapping) => mapping.internalIp));
		selectedInternalIps.value = selectedInternalIps.value.filter((ip) => existing.has(ip));
	});

async function loadElasticMappings() {
	try {
		const mappings = await useTauriCoreInvoke<ElasticMappingEntry[]>("get_all_elastic_mappings");
		elasticMappings.value = mappings;
	} catch (error) {
		toast.add({
			title: "加载映射失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	}
}

async function loadIspSummary() {
	try {
		const summary = await useTauriCoreInvoke<IspUpdateResult>("get_isp_summary");
		ispSummary.value = summary;
	} catch (error) {
		console.warn("加载运营商数据摘要失败", error);
	}
}

	async function addElasticMapping() {
		if (!newMapping.internalIp.trim() || !newMapping.elasticIp.trim()) {
			toast.add({
				title: "输入不完整",
				description: "请填写完整的内部 IP 和弹性 IP。",
				color: "warning"
			});
			return;
		}
		try {
			await useTauriCoreInvoke("add_elastic_ip_mapping", {
				request: {
					internalIp: newMapping.internalIp.trim(),
					elasticIp: newMapping.elasticIp.trim()
				}
			});
			await loadElasticMappings();
			newMapping.internalIp = "";
			newMapping.elasticIp = "";
			toast.add({
				title: "映射保存成功",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "保存映射失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function removeElasticMapping(internalIp: string) {
		try {
			await useTauriCoreInvoke("remove_elastic_ip_mapping", {
				request: { internalIp }
			});
			await loadElasticMappings();
			toast.add({
				title: "映射已删除",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "删除失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function bulkAddElasticMappings() {
		const entries = bulkMappingInput.value
			.split("\n")
			.map((line) => line.trim())
			.filter(Boolean)
			.map((line) => {
				const [left, right] = line.split("->").map((part) => part.trim());
				return left && right
					? { internalIp: left, elasticIp: right }
					: null;
			})
			.filter((entry): entry is { internalIp: string, elasticIp: string } => Boolean(entry));

		if (!entries.length) {
			toast.add({
				title: "无有效映射",
				description: "请按“内部IP -> 弹性IP”的格式填写映射。",
				color: "warning"
			});
			return;
		}

		try {
			const result = await useTauriCoreInvoke<BulkElasticResult>("bulk_add_elastic_ip_mappings", {
				request: {
					entries,
					overwriteExisting: overwriteExisting.value
				}
			});
			await loadElasticMappings();
			bulkMappingInput.value = "";
			toast.add({
				title: "批量导入完成",
				description: `新增 ${result.added} 条，更新 ${result.updated} 条，跳过 ${result.skipped} 条。`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "批量导入失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function updateIspDatabase() {
		ispUpdateLoading.value = true;
		try {
			const summary = await useTauriCoreInvoke<IspUpdateResult>("update_isp_from_github");
			ispSummary.value = summary;
			toast.add({
				title: "运营商数据更新成功",
				description: `总计 ${summary.total} 条记录`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "更新失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			ispUpdateLoading.value = false;
		}
	}

	onMounted(() => {
		void Promise.all([loadElasticMappings(), loadIspSummary()]);
	});

	function updateSelection(internalIp: string, checked: boolean) {
		if (checked) {
			if (!selectedInternalIps.value.includes(internalIp)) {
				selectedInternalIps.value.push(internalIp);
			}
		} else {
			selectedInternalIps.value = selectedInternalIps.value.filter((ip) => ip !== internalIp);
		}
	}

	function toggleSelectAll(checked: boolean) {
		if (checked) {
			const current = filteredElasticMappings.value.map((mapping) => mapping.internalIp);
			selectedInternalIps.value = Array.from(new Set([...selectedInternalIps.value, ...current]));
		} else {
			const currentSet = new Set(filteredElasticMappings.value.map((mapping) => mapping.internalIp));
			selectedInternalIps.value = selectedInternalIps.value.filter((ip) => !currentSet.has(ip));
		}
	}

	async function bulkDeleteMappings() {
		if (!selectedInternalIps.value.length) {
			toast.add({
				title: "未选择任何映射",
				description: "请先勾选需要删除的弹性 IP 映射。",
				color: "warning"
			});
			return;
		}

		try {
			for (const internalIp of selectedInternalIps.value) {
				await useTauriCoreInvoke("remove_elastic_ip_mapping", {
					request: { internalIp }
				});
			}
			await loadElasticMappings();
			selectedInternalIps.value = [];
			toast.add({
				title: "批量删除完成",
				description: "所选映射已删除。",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "批量删除失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}
</script>
