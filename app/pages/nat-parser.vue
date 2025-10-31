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
							<h3 class="text-lg font-semibold">设备类型</h3>
						</div>
					</template>
				<URadioGroup
					v-model="deviceType"
					:items="deviceOptions"
					class="flex gap-6"
				/>
			</UCard>

			<!-- 配置输入 -->
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-terminal" class="size-5" />
						<h3 class="text-lg font-semibold">NAT Server 配置</h3>
					</div>
				</template>
				<div class="space-y-4">
					<UTextarea
						v-model="configText"
						placeholder="在此粘贴 NAT Server 配置命令...&#10;&#10;华为示例：&#10;nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080&#10;&#10;H3C示例：&#10;nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080 rule 100"
						:rows="12"
						class="font-mono text-sm"
						@input="parseConfig"
					/>
					<div class="flex justify-between items-center">
						<p class="text-sm text-(--ui-text-muted)">
							支持格式：华为、H3C NAT Server 命令
						</p>
						<div class="flex gap-2">
							<UButton
								variant="outline"
								size="sm"
								@click="clearConfig"
								:disabled="!configText.trim()"
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
							<p class="text-2xl font-bold text-green-600">{{ parseResult.successEntries.length }}</p>
							<p class="text-sm text-(--ui-text-muted)">成功解析</p>
						</div>
					</UCard>
					<UCard class="bg-(--ui-bg)">
						<div class="text-center">
							<p class="text-2xl font-bold text-red-600">{{ parseResult.failedEntries.length }}</p>
							<p class="text-sm text-(--ui-text-muted)">解析失败</p>
						</div>
					</UCard>
					<UCard class="bg-(--ui-bg)">
						<div class="text-center">
							<p class="text-2xl font-bold text-blue-600">{{ totalEntries }}</p>
							<p class="text-sm text-(--ui-text-muted)">总条目数</p>
						</div>
					</UCard>
				</div>

				<!-- 成功解析的条目 -->
				<div v-if="parseResult.successEntries.length > 0">
					<div class="flex justify-between items-center mb-4">
						<h3 class="text-xl font-semibold">解析结果</h3>
						<UButton
							icon="i-lucide-download"
							@click="handleExportClick"
							:disabled="parseResult.successEntries.length === 0"
						>
							导出到 Excel
						</UButton>
					</div>
					<UCard class="bg-(--ui-bg)">
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-(--ui-border)">
										<th class="text-left p-3 font-medium">名称</th>
										<th class="text-left p-3 font-medium">协议</th>
										<th class="text-left p-3 font-medium">全局IP</th>
										<th class="text-left p-3 font-medium">全局端口</th>
										<th class="text-left p-3 font-medium">内部IP</th>
										<th class="text-left p-3 font-medium">内部端口</th>
										<th v-if="deviceType === 'h3c'" class="text-left p-3 font-medium">VRRP</th>
										<th v-if="deviceType === 'h3c'" class="text-left p-3 font-medium">规则</th>
										<th v-if="deviceType === 'h3c'" class="text-left p-3 font-medium">描述</th>
									</tr>
								</thead>
								<tbody>
									<tr
										v-for="(entry, index) in parseResult.successEntries"
										:key="index"
										class="border-b border-(--ui-border) hover:bg-(--ui-bg-muted)"
									>
										<td class="p-3 font-mono">{{ entry.name }}</td>
										<td class="p-3">{{ entry.protocol }}</td>
										<td class="p-3 font-mono">{{ entry.globalIp }}</td>
										<td class="p-3 font-mono">{{ entry.globalPort }}</td>
										<td class="p-3 font-mono">{{ entry.insideIp }}</td>
										<td class="p-3 font-mono">{{ entry.insidePort }}</td>
										<td v-if="deviceType === 'h3c'" class="p-3">{{ entry.vrrp }}</td>
										<td v-if="deviceType === 'h3c'" class="p-3 font-mono">{{ entry.rule }}</td>
										<td v-if="deviceType === 'h3c'" class="p-3">{{ entry.description }}</td>
									</tr>
								</tbody>
							</table>
						</div>
					</UCard>
				</div>

				<!-- 解析失败的条目 -->
				<div v-if="parseResult.failedEntries.length > 0">
					<h3 class="text-xl font-semibold mb-4 text-red-600">解析失败条目</h3>
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
						<h3 class="text-lg font-semibold">使用说明</h3>
					</div>
				</template>
				<div class="space-y-4 text-sm">
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">支持设备</p>
							<p class="text-(--ui-text-muted)">华为防火墙、H3C 防火墙的 NAT Server 配置命令</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">华为格式</p>
							<p class="text-(--ui-text-muted)">nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">H3C 格式</p>
							<p class="text-(--ui-text-muted)">nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080 rule 100</p>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<Icon name="i-lucide-check" class="size-4 text-green-500 mt-0.5" />
						<div>
							<p class="font-medium">导出功能</p>
							<p class="text-(--ui-text-muted)">支持将解析结果导出为 Excel 文件，便于后续处理</p>
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

	type NatEntry = {
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
	};

	type NatParseResult = {
		successEntries: NatEntry[]
		failedEntries: string[]
		deviceType: string
	};

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
		parseConfig();
	};

	const isTauriEnvironment = () => {
		if (typeof window === "undefined") return false;
		const tauriGlobal = (window as any).__TAURI__;
		const tauriInternal = (window as any).__TAURI_INTERNALS__;
		if (tauriGlobal || tauriInternal) {
			return true;
		}
		const ua = typeof navigator !== "undefined" ? navigator.userAgent?.toLowerCase?.() ?? "" : "";
		return ua.includes("tauri");
	};

	// 处理导出点击事件
	const handleExportClick = () => {
		console.log("导出按钮被点击，成功条目数量:", parseResult.value.successEntries.length);
		exportToExcel();
	};

	// 导出到 Excel
	const exportToExcel = async () => {
		console.log("导出功能被触发");
		if (parseResult.value.successEntries.length === 0) {
			console.log("没有成功条目，导出终止");
			toast.add({
				title: "暂无可导出的数据",
				description: "请先粘贴并解析 NAT 配置。",
				icon: "i-lucide-info"
			});
			return;
		}

		console.log("开始创建 Excel 工作簿");

		try {
			const XLSX = await import("xlsx");
			console.log("成功导入 XLSX 库");

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
			parseResult.value.successEntries.forEach(entry => {
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
				parseResult.value.failedEntries.forEach(entry => {
					failedData.push([entry]);
				});
				const wsFailed = XLSX.utils.aoa_to_sheet(failedData);
				XLSX.utils.book_append_sheet(wb, wsFailed, "解析失败");
			}

			// 生成 Excel 文件
			const excelBuffer = XLSX.write(wb, { bookType: 'xlsx', type: 'array' });
			const blob = new Blob([excelBuffer], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });

			const fileName = `nat_config_${deviceType.value}_${new Date().toISOString().slice(0, 10)}.xlsx`;
			console.log("Excel 文件创建成功，文件名:", fileName);

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
		console.log("文件名:", fileName);
		console.log("是否在 Tauri 环境:", isTauriEnvironment());

		if (isTauriEnvironment()) {
			try {
				const { writeFile } = await import("@tauri-apps/plugin-fs");
				const { downloadDir, join } = await import("@tauri-apps/api/path");

				const downloadsPath = await downloadDir();
				const fullPath = await join(downloadsPath, fileName);
				const arrayBuffer = await blob.arrayBuffer();
				await writeFile(fullPath, new Uint8Array(arrayBuffer));
				console.log("文件保存成功到下载目录:", fullPath);

				toast.add({
					title: "导出成功",
					description: `文件已保存至: ${fullPath}`,
					icon: "i-lucide-check-circle",
					timeout: 8000,
					actions: [
						{
							label: "打开位置",
							click: async () => {
								try {
									const { Command } = await import("@tauri-apps/plugin-shell");
									const platform = navigator.platform.toLowerCase();
									if (platform.includes("win")) {
										const windowsPath = fullPath.replace(/\//g, "\\");
										const command = new Command("open-explorer", [`/select,"${windowsPath}"`]);
										await command.execute();
									} else if (platform.includes("mac")) {
										const command = new Command("open-folder", ["-R", fullPath]);
										await command.execute();
									} else {
										const command = new Command("exec-sh", [`xdg-open "${downloadsPath}"`]);
										await command.execute();
									}
								} catch (openError) {
									console.error("打开文件位置失败:", openError);
									toast.add({
										title: "无法打开文件位置",
										description: `请手动前往: ${downloadsPath}`,
										icon: "i-lucide-alert-triangle",
										timeout: 10000
									});
								}
							}
						}
					]
				});
			} catch (error) {
				console.error("Tauri保存失败:", error);
				toast.add({
					title: "导出失败",
					description: "保存文件时发生错误，请重试。",
					color: "red",
					icon: "i-lucide-alert-triangle"
				});
			}
		} else {
			console.log("使用浏览器下载 API");
			try {
				console.log("准备下载 Excel 文件，大小:", blob.size);

				// 创建下载链接
				const url = URL.createObjectURL(blob);
				const link = document.createElement('a');
				link.href = url;
				link.download = fileName;
				link.style.display = 'none';
				document.body.appendChild(link);
				link.click();
				document.body.removeChild(link);
				URL.revokeObjectURL(url);

				console.log("Excel 文件下载触发");

				toast.add({
					title: "导出成功",
					description: `文件已下载: ${fileName}`,
					icon: "i-lucide-check-circle",
					timeout: 8000,
					actions: [{
						label: "复制路径",
						click: async () => {
							try {
								// 在浏览器环境下，复制文件名到剪贴板
								await navigator.clipboard.writeText(fileName);
								// 显示一个简单的提示
								toast.add({
									title: "文件名已复制",
									description: "请在下载目录中查找此文件",
									timeout: 3000
								});
							} catch (copyError) {
								console.log("无法复制文件名");
							}
						}
					}]
				});
			} catch (error) {
				console.error("浏览器下载失败:", error);
				toast.add({
					title: "导出失败",
					description: "浏览器下载失败，请重试。",
					color: "red",
					icon: "i-lucide-alert-triangle"
				});
			}
		}
	};

	// 监听设备类型变化
	watch(deviceType, () => {
		parseConfig();
	});
</script>
