import type { TeraTemplateAnalysis } from "~/types/template-batch";

export type VariableTags = {
	loop: boolean
	conditional: boolean
	defaultable: boolean
	formatting: boolean
};

export type TemplateHintOptions = {
	iterableFields?: TeraTemplateAnalysis["iterableFields"]
	sampleValues?: TeraTemplateAnalysis["sampleValues"]
	defaultFallbacks?: TeraTemplateAnalysis["defaultFallbacks"]
	fallbackProviders?: Map<string, string[]>
};

const GENERIC_SAMPLE_SUFFIX = " 示例值";
const FORMATTED_SAMPLE_SUFFIX = " 示例值（自动格式化）";

export function buildSampleValue(
	variable: string,
	tags: VariableTags,
	options: TemplateHintOptions
): string {
	const sampleValues = options.sampleValues ?? {};
	const iterableFields = options.iterableFields ?? {};
	const defaultFallbacks = options.defaultFallbacks ?? {};

	if (tags.loop) {
		return buildIterableSample(variable, iterableFields, sampleValues);
	}

	const values = sampleValues[variable] ?? [];
	const defaultHint = defaultFallbacks[variable];

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
		return `${variable}${FORMATTED_SAMPLE_SUFFIX}`;
	}

	const providerMessage = getProviderMessage(variable, options.fallbackProviders);
	if (providerMessage) {
		return providerMessage;
	}

	return buildScalarHint(variable);
}

export function shouldShowSample(variable: string, sample: string): boolean {
	if (!sample) return false;
	const generic = `${variable}${GENERIC_SAMPLE_SUFFIX}`;
	const formatted = `${variable}${FORMATTED_SAMPLE_SUFFIX}`;
	return sample !== generic && sample !== formatted;
}

export function formatExampleList(values: string[]): string {
	if (!values || !values.length) return "示例缺失";
	const preview = values.slice(0, 3).join(" / ");
	return values.length > 3 ? `${preview} / ...` : preview;
}

function buildIterableSample(
	variable: string,
	iterableFields: Record<string, string[]>,
	sampleValues: Record<string, string[]>
): string {
	let childFields = iterableFields[variable] ?? [];
	if (!childFields.length) {
		childFields = inferDefaultChildFields(variable);
	}
	if (!childFields.length) {
		return buildScalarIterableSample(variable);
	}

	const sampleCount = 3;
	const entries: string[] = [];
	for (let index = 0; index < sampleCount; index += 1) {
		const fields = childFields.map((field) => {
			const value = sampleIterableFieldValue(field, variable, index, sampleValues);
			return `"${field}":"${value}"`;
		});
		entries.push(`{${fields.join(",")}}`);
	}
	return `[${entries.join(",")}]`;
}

function buildScalarIterableSample(name: string): string {
	const numeric = inferNumericExample(name);
	if (numeric) {
		const base = Number.parseInt(numeric, 10);
		if (!Number.isNaN(base)) {
			return `["${base}","${base + 1}","${base + 10}"]`;
		}
		return `["${numeric}","${numeric}_2","${numeric}_10"]`;
	}
	return `["${name}1","${name}2","${name}10"]`;
}

function buildScalarHint(variable: string): string {
	const startEndHint = inferRangeHint(variable);
	if (startEndHint) {
		return startEndHint;
	}
	const numeric = inferNumericExample(variable);
	if (numeric) {
		const descriptor = describeNumericVariable(variable);
		return `${descriptor}（示例 ${numeric}）`;
	}
	const ip = inferIpExample(variable);
	if (ip) {
		return `${describeIpVariable(variable)}（示例 ${ip}）`;
	}
	return `${variable}${GENERIC_SAMPLE_SUFFIX}`;
}

function sampleValueForField(
	field: string,
	parent: string,
	sampleValues: Record<string, string[]>
): string {
	const compound = `${parent}.${field}`;
	const fromAnalysis = sampleValues[compound];
	if (fromAnalysis && fromAnalysis.length) {
		return fromAnalysis[0];
	}
	const numeric = inferNumericExample(field);
	if (numeric) return numeric;
	const ip = inferIpExample(field);
	if (ip) return ip;
	const normalizedField = normalizedName(field).toLowerCase();
	if (normalizedField.includes("name")) {
		return `${parent}名称`;
	}
	if (normalizedField.includes("desc") || normalizedField.includes("remark")) {
		return "示例描述";
	}
	return `${field}示例`;
}

function sampleIterableFieldValue(
	field: string,
	parent: string,
	index: number,
	sampleValues: Record<string, string[]>
): string {
	const compound = `${parent}.${field}`;
	const fromAnalysis = sampleValues[compound];
	if (fromAnalysis && fromAnalysis.length) {
		const sample = fromAnalysis[Math.min(index, fromAnalysis.length - 1)];
		if (sample) return sample;
	}
	const numeric = inferNumericExample(field);
	if (numeric) {
		const base = Number.parseInt(numeric, 10);
		if (!Number.isNaN(base)) {
			const offsets = [0, 1, 9];
			const offset = offsets[index] ?? index;
			return String(base + offset);
		}
		return numeric;
	}
	const ip = inferIpExample(field);
	if (ip) {
		return incrementIp(ip, index);
	}
	const base = sampleValueForField(field, parent, sampleValues);
	return index === 0 ? base : `${base}_${index + 1}`;
}

function inferNumericExample(name: string): string | null {
	const lowered = normalizedName(name).toLowerCase();
	if (lowered.includes("vlan")) {
		if (lowered.includes("start")) return "100";
		if (lowered.includes("end")) return "200";
		return "10";
	}
	if (lowered.includes("start")) {
		return "1";
	}
	if (lowered.includes("end")) {
		return "10";
	}
	if (lowered.endsWith("_id") || lowered.endsWith("id")) {
		return "1";
	}
	if (lowered.endsWith("_number") || lowered.endsWith("_no")) {
		return "42";
	}
	if (lowered.endsWith("_count") || lowered.endsWith("_size")) {
		return "2";
	}
	if (lowered.includes("port")) {
		return "48";
	}
	if (lowered.includes("slot")) {
		return "2";
	}
	return null;
}

function inferIpExample(name: string): string | null {
	const lowered = normalizedName(name).toLowerCase();
	if (lowered.includes("gateway")) {
		return "10.0.0.1";
	}
	if (lowered.includes("loopback")) {
		return "192.168.255.1";
	}
	if (lowered.includes("ip")) {
		return "10.0.0.1";
	}
	if (lowered.includes("mask") || lowered.includes("netmask")) {
		return "255.255.255.0";
	}
	return null;
}

function inferRangeHint(name: string): string | null {
	const lowered = normalizedName(name).toLowerCase();
	if (lowered.includes("start") && lowered.includes("vlan")) {
		return "起始 VLAN ID（示例 100）";
	}
	if (lowered.includes("end") && lowered.includes("vlan")) {
		return "结束 VLAN ID（示例 200）";
	}
	if (lowered.includes("start")) {
		return "起始值（示例 1）";
	}
	if (lowered.includes("end")) {
		return "结束值（示例 10）";
	}
	return null;
}

function describeNumericVariable(name: string): string {
	const lowered = normalizedName(name).toLowerCase();
	if (lowered.includes("vlan")) {
		return lowered.includes("range") ? "VLAN 范围" : "VLAN ID";
	}
	if (lowered.endsWith("id")) {
		return "标识 ID";
	}
	if (lowered.includes("count")) {
		return "数量";
	}
	return "数值";
}

function describeIpVariable(name: string): string {
	const lowered = normalizedName(name).toLowerCase();
	if (lowered.includes("gateway")) {
		return "网关 IP";
	}
	if (lowered.includes("loopback")) {
		return "Loopback IP";
	}
	if (lowered.includes("mask") || lowered.includes("netmask")) {
		return "子网掩码";
	}
	return "IP 地址";
}

function inferDefaultChildFields(name: string): string[] {
	const lowered = normalizedName(name).toLowerCase();
	if (lowered.includes("vlan")) {
		return ["id"];
	}
	if (lowered.endsWith("s")) {
		return ["id"];
	}
	return [];
}

function incrementIp(ip: string, index: number): string {
	const octets = ip.split(".").map((segment) => Number.parseInt(segment, 10));
	if (octets.length !== 4 || octets.some((segment) => Number.isNaN(segment))) {
		return ip;
	}
	const result = [...octets];
	result[3] = Math.max(1, result[3] + index);
	return result.join(".");
}

function normalizedName(name: string): string {
	const segments = name.split(".");
	return segments[segments.length - 1] ?? name;
}

function getProviderMessage(variable: string, providers?: Map<string, string[]>): string | null {
	if (!providers) return null;
	const consumers = providers.get(variable);
	if (!consumers || !consumers.length) return null;
	if (consumers.length === 1) {
		return `供 ${consumers[0]} 默认引用，请填写实际值`;
	}
	const preview = consumers.slice(0, 3).join(" / ");
	return consumers.length > 3
		? `供 ${preview} / ... 默认引用，请填写实际值`
		: `供 ${preview} 默认引用，请填写实际值`;
}
