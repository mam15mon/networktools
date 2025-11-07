<template>
	<LayoutTile
		title="防火墙 NAT 配置解析"
		description="解析华为和 H3C 防火墙的 NAT Server 配置命令，支持多种格式，可导出为 Excel 表格。"
	>
		<div class="space-y-8">
			<!-- 设备类型选择 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-settings" class="size-5" />
						<h3 class="text-lg font-semibold">
							设备类型
						</h3>
					</div>
				</template>
				<URadioGroup
					v-model="deviceType"
					:items="deviceOptions"
					orientation="horizontal"
					class="flex flex-wrap gap-6"
				/>
			</UCard>

			<!-- 配置输入 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-terminal" class="size-5" />
						<h3 class="text-lg font-semibold">
							NAT Server 配置
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<UTextarea
						v-model="configText"
						placeholder="在此粘贴 NAT Server 配置命令...&#10;&#10;华为示例：&#10;nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080&#10;&#10;H3C示例：&#10;nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080 rule 100"
						:rows="12"
						class="font-mono text-sm"
						@input="parseConfig"
						@paste="handlePaste"
					/>
					<div class="flex justify-between items-center">
						<p class="text-sm text-(--ui-text-muted)">
							支持格式：华为、H3C NAT Server 命令
						</p>
						<div class="flex gap-2">
							<UButton
								variant="outline"
								size="sm"
								:disabled="!configText.trim()"
								@click="clearConfig"
							>
								清空
							</UButton>
							<UButton
								variant="outline"
								size="sm"
								@click="loadExample"
							>
								加载示例
							</UButton>
						</div>
					</div>
				</div>
			</UCard>

			<!-- 解析结果 -->
			<div v-if="parseResult.successEntries.length > 0 || parseResult.failedEntries.length > 0" class="space-y-6">
				<!-- 统计信息 -->
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					<UCard class="bg-(--ui-bg)">
						<div class="text-center">
							<p class="text-2xl font-bold text-green-600">
								{{ parseResult.successEntries.length }}
							</p>
							<p class="text-sm text-(--ui-text-muted)">
								成功解析
							</p>
						</div>
					</UCard>
					<UCard class="bg-(--ui-bg)">
						<div class="text-center">
							<p class="text-2xl font-bold text-red-600">
								{{ parseResult.failedEntries.length }}
							</p>
							<p class="text-sm text-(--ui-text-muted)">
								解析失败
							</p>
						</div>
					</UCard>
					<UCard class="bg-(--ui-bg)">
						<div class="text-center">
							<p class="text-2xl font-bold text-blue-600">
								{{ totalEntries }}
							</p>
							<p class="text-sm text-(--ui-text-muted)">
								总条目数
							</p>
						</div>
					</UCard>
				</div>

				<!-- 成功解析的条目 -->
				<div v-if="parseResult.successEntries.length > 0">
					<div class="flex justify-between items-center mb-4">
						<h3 class="text-xl font-semibold">
							解析结果
						</h3>
						<UButton
							icon="i-lucide-download"
							:disabled="parseResult.successEntries.length === 0"
							@click="handleExportClick"
						>
							导出到 Excel
						</UButton>
					</div>
					<UCard class="bg-(--ui-bg)">
						<UTable
							:columns="successColumns"
							:data="successRows"
							class="w-full text-sm"
							:ui="{ td: { base: 'align-top' } }"
						/>
					</UCard>
				</div>

				<!-- 解析失败的条目 -->
				<div v-if="parseResult.failedEntries.length > 0">
					<h3 class="text-xl font-semibold mb-4 text-red-600">
						解析失败条目
					</h3>
					<UCard class="bg-(--ui-bg-muted) border-(--ui-border)">
						<div class="space-y-2">
							<div
								v-for="(entry, index) in parseResult.failedEntries"
								:key="index"
								class="p-3 font-mono text-sm text-(--ui-text) bg-(--ui-bg) border border-(--ui-border) rounded"
							>
								<span class="text-red-600 font-medium">解析失败:</span> {{ entry }}
							</div>
						</div>
					</UCard>
				</div>
			</div>

			<!-- 使用说明 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-help-circle" class="size-5" />
						<h3 class="text-lg font-semibold">
							使用说明
						</h3>
					</div>
				</template>
				<div class="space-y-4 text-sm">
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								支持设备
							</p>
							<p class="text-(--ui-text-muted)">
								华为防火墙、H3C 防火墙的 NAT Server 配置命令
							</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								华为格式
							</p>
							<p class="text-(--ui-text-muted)">
								nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080
							</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								H3C 格式
							</p>
							<p class="text-(--ui-text-muted)">
								nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080 rule 100
							</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">
								导出功能
							</p>
							<p class="text-(--ui-text-muted)">
								支持将解析结果导出为 Excel 文件，便于后续处理
							</p>
						</div>
					</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	definePageMeta({
		name: "NAT 配置解析",
		icon: "lucide-shield",
		description: "防火墙 NAT 配置解析工具",
		category: "firewall"
	});

	interface NatEntry {
		name: string
		protocol: string
		globalIp: string
		globalPort: string
		insideIp: string
		insidePort: string
		vrrp?: string
		rule?: string
		description?: string
		command: string
	}

	interface NatParseResult {
		successEntries: NatEntry[]
		failedEntries: string[]
		deviceType: string
	}

	const deviceOptions = [
		{ label: "华为防火墙", value: "huawei" },
		{ label: "H3C 防火墙", value: "h3c" }
	] as const;
	type DeviceType = (typeof deviceOptions)[number]["value"];
	const deviceType = ref<DeviceType>(deviceOptions[0].value);
	const configText = ref("");
	const parseResult = ref<NatParseResult>({
		successEntries: [],
		failedEntries: [],
		deviceType: "huawei"
	});
	const toast = useToast();

	const totalEntries = computed(() =>
		parseResult.value.successEntries.length + parseResult.value.failedEntries.length
	);

	const successColumns = computed(() => {
		const columns = [
			{ accessorKey: "name", header: "名称", id: "name" },
			{ accessorKey: "protocol", header: "协议", id: "protocol" },
			{ accessorKey: "globalIp", header: "全局IP", id: "globalIp" },
			{ accessorKey: "globalPort", header: "全局端口", id: "globalPort" },
			{ accessorKey: "insideIp", header: "内部IP", id: "insideIp" },
			{ accessorKey: "insidePort", header: "内部端口", id: "insidePort" }
		];
		if (deviceType.value === "h3c") {
			columns.push(
				{ accessorKey: "vrrp", header: "VRRP", id: "vrrp" },
				{ accessorKey: "rule", header: "规则", id: "rule" },
				{ accessorKey: "description", header: "描述", id: "description" }
			);
		}
		return columns;
	});

	const successRows = computed(() =>
		parseResult.value.successEntries.map((entry, index) => ({
			...entry,
			rowKey: `${entry.name}-${index}`
		}))
	);

	// 处理粘贴事件
	const handlePaste = (event: ClipboardEvent) => {
		// 延迟一点确保粘贴内容已经更新到 v-model
		setTimeout(() => {
			parseConfig();
		}, 100);
	};

	// 解析配置
	const parseConfig = async () => {
		if (!configText.value.trim()) {
			parseResult.value = {
				successEntries: [],
				failedEntries: [],
				deviceType: deviceType.value
			};
			return;
		}

		try {
			const result = await useTauriCoreInvoke<NatParseResult>("parse_nat_config", {
				text: configText.value,
				deviceType: deviceType.value
			});
			parseResult.value = result;
		} catch (error) {
			console.error("解析 NAT 配置失败:", error);
			parseResult.value = {
				successEntries: [],
				failedEntries: [configText.value],
				deviceType: deviceType.value
			};
		}
	};

	// 清空配置
	const clearConfig = () => {
		configText.value = "";
		parseResult.value = {
			successEntries: [],
			failedEntries: [],
			deviceType: deviceType.value
		};
	};

	// 加载示例
	const loadExample = () => {
		if (deviceType.value === "huawei") {
			configText.value = `nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080
nat server protocol udp global 202.100.10.2 53 inside 192.168.1.101 53
nat server "web-server" protocol tcp global 202.100.10.3 www inside 192.168.1.200 80
nat server protocol tcp global 202.100.10.4 443 inside 192.168.1.200 8443`;
		} else {
			configText.value = `nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080 rule 100
nat server protocol udp global 202.100.10.2 53 inside 192.168.1.101 53 rule 101
nat server protocol tcp global 202.100.10.3 443 inside 192.168.1.200 8443 rule 102 description Web server
nat server protocol tcp global 202.100.10.4 80 inside 192.168.1.200 80 rule 103 vrrp 1`;
		}

		// 加载示例后自动解析
		parseConfig();
	};



	// 处理导出点击事件
	const handleExportClick = () => {
		exportToExcel();
	};

	// 导出到 Excel
	const exportToExcel = async () => {
		if (parseResult.value.successEntries.length === 0) {
			toast.add({
				title: "暂无可导出的数据",
				description: "请先粘贴并解析 NAT 配置。",
				icon: "i-lucide-info"
			});
			return;
		}

		try {
			const XLSX = await import("xlsx");

			// 创建工作簿
			const wb = XLSX.utils.book_new();

			// 准备成功条目的数据
			const successData = [];

			// 表头
			const headers = ["名称", "协议", "全局IP", "全局端口", "内部IP", "内部端口"];
			if (deviceType.value === "h3c") {
				headers.push("VRRP", "规则", "描述");
			}
			headers.push("原始命令");
			successData.push(headers);

			// 数据行
			parseResult.value.successEntries.forEach((entry) => {
				const row = [
					entry.name,
					entry.protocol,
					entry.globalIp,
					entry.globalPort,
					entry.insideIp,
					entry.insidePort
				];

				if (deviceType.value === "h3c") {
					row.push(
						entry.vrrp || "",
						entry.rule || "",
						entry.description || ""
					);
				}

				row.push(entry.command);
				successData.push(row);
			});

			// 创建成功条目工作表
			const wsSuccess = XLSX.utils.aoa_to_sheet(successData);
			XLSX.utils.book_append_sheet(wb, wsSuccess, "解析结果");

			// 如果有失败条目，创建失败条目工作表
			if (parseResult.value.failedEntries.length > 0) {
				const failedData = [["解析失败条目"]];
				parseResult.value.failedEntries.forEach((entry) => {
					failedData.push([entry]);
				});
				const wsFailed = XLSX.utils.aoa_to_sheet(failedData);
				XLSX.utils.book_append_sheet(wb, wsFailed, "解析失败");
			}

			// 生成 Excel 文件
			const excelBuffer = XLSX.write(wb, { bookType: "xlsx", type: "array" });
			const blob = new Blob([excelBuffer], { type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" });

			const fileName = `nat_config_${deviceType.value}_${new Date().toISOString().slice(0, 10)}.xlsx`;

			// 保存文件
			await saveFile(blob, fileName);
		} catch (error) {
			console.error("生成 Excel 文件失败:", error);
			toast.add({
				title: "导出失败",
				description: "生成 Excel 文件时出现问题，请重试。",
				color: "red",
				icon: "i-lucide-alert-triangle"
			});
		}
	};

	// 保存文件的通用函数
	const saveFile = async (blob: Blob, fileName: string) => {
		const { save } = await import("@tauri-apps/plugin-dialog");
		const targetPath = await save({
			defaultPath: fileName,
			filters: [{ name: "Excel", extensions: ["xlsx"] }]
		});
		if (!targetPath) {
			return;
		}

		const arrayBuffer = await blob.arrayBuffer();
		const { writeFile } = await import("@tauri-apps/plugin-fs");
		await writeFile(targetPath, new Uint8Array(arrayBuffer));


		toast.add({
			title: "导出成功",
			description: `文件已保存到 ${targetPath}`,
			icon: "i-lucide-check-circle",
			timeout: 4000
		});
	};

	// 监听设备类型变化
	watch(deviceType, () => {
		if (configText.value.trim()) {
			void parseConfig();
		} else {
			parseResult.value = {
				successEntries: [],
				failedEntries: [],
				deviceType: deviceType.value
			};
		}
	});
</script>
