<template>
	<UDrawer v-model:open="open" title="Tera 模板教学" :description="`常用语法与示例 (网工实战版)`" direction="right">
		<UButton
			variant="soft"
			color="success"
			icon="i-lucide-graduation-cap"
			@click="open = true"
		>
			Tera 模板教学
		</UButton>
		<template #body>
			<div class="p-4 space-y-6 text-sm leading-relaxed text-(--ui-text-muted)">
				<div class="flex flex-col gap-2 md:flex-row md:items-center md:gap-4 p-4 bg-green-50 border border-green-200 rounded-xl text-green-800">
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-badge-check" class="size-5 text-green-600" />
						<p class="text-base font-semibold">Tera 入门指南 (网络工程师版)</p>
					</div>
				</div>
				<section>
					<UPageList>
						<header class="space-y-1">
							<h3 class="text-base font-semibold text-green-800">入门：变量长什么样</h3>
							<p>模板中所有变量都由英文字母、数字与下划线组成，推荐使用 <code>snake_case</code>，且不能包含中文、空格或运算符号。</p>
						</header>
						<div class="space-y-2">
							<div class="flex items-center gap-2"><UKbd>Excel 列名</UKbd><span class="text-sm">会被映射成同名变量，如列 <code>device_hostname</code> → <code v-pre>{{ device_hostname }}</code></span></div>
							<div class="flex items-center gap-2"><UKbd>自定义变量</UKbd><span class="text-sm">用 <code v-pre>{% set core_id = device.id + 1 %}</code> 定义，仅在当前作用域可用</span></div>
							<div class="flex items-center gap-2"><UKbd>组合访问</UKbd><span class="text-sm">多层数据用点号：<code v-pre>{{ device.ip.loopback }}</code></span></div>
						</div>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.variableBasics" />
						<UAlert color="warning" variant="soft" class="mt-3">
							<template #description>变量名仅支持 <code>a-z</code> / <code>A-Z</code> / <code>0-9</code> / <code>_</code>，<strong>不要</strong>出现中文字符、加减号、空格或其它符号，否则解析失败。</template>
						</UAlert>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">变量与过滤器</h3>
						<p>使用 <code v-pre>{{ variable | filter }}</code> 组合变量与过滤逻辑。</p>
						<div class="space-y-2">
							<div class="flex items-center gap-2"><UKbd>upper</UKbd><span class="text-sm">全部大写，常用于接口描述</span></div>
							<div class="flex items-center gap-2"><UKbd>capitalize</UKbd><span class="text-sm">句首大写，如 "core router" → "Core router"</span></div>
							<div class="flex items-center gap-2"><UKbd>trim / trim_start / trim_end</UKbd><span class="text-sm text-(--ui-text-muted)">清除模板或 Excel 中的多余空白</span></div>
							<div class="flex items-center gap-2"><UKbd>replace</UKbd><span class="text-sm"><code>from=" ", to="_"</code> 替换空格，生成合法标识符</span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<header class="space-y-1">
							<h3 class="text-base font-semibold text-green-800">核心：空白控制 (网工必看)</h3>
							<p>
								默认情况下，Tera 模板中的换行会保留。但在网络配置中，我们希望
								<code v-pre>{% for %}</code
								> 循环本身<strong>不要</strong>产生多余的空行。
							</p>
						</header>
						<p class="mt-2 text-sm">
							使用 <code v-pre>{%- ... -%}</code
							> 或 <code v-pre>{{- ... -}}</code
							> (注意那个减号 <code v-pre">-</code
							>) 来“吃掉”前后的空白（包括换行符）。
						</p>
						<TemplateBatchCodeSnippet
							class="mt-2"
							:code="snippets.whitespace"
						/>
						<UAlert color="warning" variant="soft" class="mt-3">
							<template #description
								><strong>强烈推荐</strong>：在 <code v-pre>{% for %}</code
								>、<code v-pre>{% if %}</code
								> 和 <code v-pre>{% endfor %}</code
								>
								这类控制标签上总是使用 <code v-pre>{%- ... -%}</code
								>，避免配置中出现意外的空行。</template
							>
						</UAlert>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">模板注释</h3>
						<p>
							使用 <code v-pre>{# ... #}</code
							> 添加的注释<strong>不会</strong>出现在最终生成的配置中，仅用于模板维护。
						</p>
						<TemplateBatchCodeSnippet
							class="mt-2"
							:code="snippets.comments"
						/>
						<div class="mt-2 space-y-2 text-sm">
							<div class="flex items-center gap-2">
								<UKbd>{# 模板注释 #}</UKbd
								><span class="text-(--ui-text-muted)"
									>用于解释模板逻辑，最终输出时消失</span
								>
							</div>
							<div class="flex items-center gap-2">
								<UKbd>! 或 #</UKbd
								><span>网设配置注释，会正常输出到配置中</span>
							</div>
						</div>
					</UPageList>
				</section>

				<section>
					<h3 class="text-base font-semibold text-green-800">循环</h3>
					<p>遍历数组/列表时使用 <code v-pre>{% for item in items %}</code>。</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.loop" />
					<p class="mt-2 text-xs">Excel 中需为 <code>vlans</code> 列提供 JSON 数组，例如：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.vlanArray" />
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">条件</h3>
						<p>可使用条件语句进行分支控制：</p>
						<div class="space-y-2">
							<div class="flex items-center gap-2"><UKbd>{% if %}</UKbd><span class="text-sm">条件为真时执行</span></div>
							<div class="flex items-center gap-2"><UKbd>{% elif %}</UKbd><span class="text-sm">额外条件检查</span></div>
							<div class="flex items-center gap-2"><UKbd>{% else %}</UKbd><span class="text-sm">以上条件都不满足时执行</span></div>
							<div class="flex items-center gap-2"><UKbd>{% endif %}</UKbd><span class="text-sm text-(--ui-text-muted)">结束条件块</span></div>
						</div>
						<TemplateBatchCodeSnippet class="mt-3" :code="snippets.condition" />
						<UAlert color="neutral" variant="subtle" class="mt-3">
							<template #description>Excel 中的 <code>device.role</code> 建议填 <code>core</code>、<code>distribution</code> 等枚举值</template>
						</UAlert>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">进阶条件：`and`, `or`, `not`</h3>
						<p>
							使用逻辑操作符构建更复杂的 <code v-pre>{% if %}</code
							> 判断，灵活处理不同场景。
						</p>
						<TemplateBatchCodeSnippet
							class="mt-2"
							:code="snippets.advCondition"
						/>
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2">
								<UKbd>and</UKbd
								><span class="text-sm">两个条件必须同时满足</span>
							</div>
							<div class="flex items-center gap-2">
								<UKbd>or</UKbd><span class="text-sm">满足任意一个条件即可</span>
							</div>
							<div class="flex items-center gap-2">
								<UKbd>not</UKbd
								><span class="text-sm text-(--ui-text-muted)"
									>条件不成立时执行</span
								>
							</div>
						</div>
					</UPageList>
				</section>

				<section>
					<UPageList>
					<header class="space-y-1">
						<h3 class="text-base font-semibold text-green-800">批量生成接口</h3>
						<p>结合 <code>range</code> 与循环，可快速生成连续端口配置：</p>
					</header>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.portsSingleCard" />
					<p class="text-xs mt-3">如果需要跨板卡（如 1/0/x 与 3/0/x），可以嵌套循环：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.portsMultiCard" />
					<p class="text-xs text-(--ui-text-muted)">
						技巧：循环体内部使用普通 <code v-pre>{% ... %}</code> 保留行间距，结束标签可以写成 <code v-pre>{%- endfor %}</code> 来清理多余空行。
					</p>
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>device.port_count</UKbd><span class="text-sm">填写端口总数，如 <code>42</code> 生成 1-42 连续接口</span></div>
							<div class="flex items-center gap-2"><UKbd>slot_range</UKbd><span class="text-sm text-(--ui-text-muted)">自定义板卡列表或上下限，适合 1/0/x ~ 3/0/x 这类结构</span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">Default 回退</h3>
						<p>使用 <code v-pre>{{ value | default(value=fallback) }}</code> 可在 Excel 留空时使用备用字段。</p>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.defaultSpeed" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>interface.speed</UKbd><span class="text-sm">主要字段，为空时使用回退值</span></div>
							<div class="flex items-center gap-2"><UKbd>interface.max_speed</UKbd><span class="text-sm text-(--ui-text-muted)">备用字段</span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<h3 class="text-base font-semibold text-green-800">宏：复用接口/邻居配置</h3>
					<p>在同一个模板文件中使用 <code v-pre>{% macro %}</code>，可以把重复的接口或 BGP 邻居片段封装起来，按需调用：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.macro" />
					<p class="mt-2 text-xs">调用示例（同一个宏可在多处复用）：</p>
					<TemplateBatchCodeSnippet class="mt-2" :code="snippets.macroCall" />
					<UAlert color="neutral" variant="subtle" class="mt-3">
						<template #description>例如需要为多台 PE 设备生成相同格式的接口/BGP 邻居配置时，只需在 Excel 中列出参数，模板里调用宏即可减少复制粘贴</template>
					</UAlert>
				</section>
				
				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">数学运算</h3>
					<p>
						Tera 支持基础的数学运算
						<code v-pre>+</code>, <code v-pre>-</code>,
						<code v-pre>*</code>, <code v-pre>/</code>, <code v-pre>%</code> (取余)。
					</p>
					<p class="text-xs text-green-700">
						提示：<code v-pre>+</code> 只做数学运算；拼接字符串（例如组装 IP）请使用
						<code v-pre>~</code>，并用 <code v-pre>| format</code> 将整数转成字符串。
					</p>
						<TemplateBatchCodeSnippet
							class="mt-2"
							:code="snippets.math"
						/>
						<p class="mt-2 text-xs">
							例如：计算 Vlan-Interface
							的IP地址。假设Excel中只提供了VLAN ID和网关偏移量。
						</p>
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2">
								<UKbd>device.vlan_base</UKbd
								><span class="text-sm">如 <code>100</code></span>
							</div>
							<div class="flex items-center gap-2">
								<UKbd>device.gw_offset</UKbd
								><span class="text-sm text-(--ui-text-muted)"
									>如 <code>1</code> (即 .1 做网关)</span
								>
							</div>
						</div>
					</UPageList>
				</section>
				
				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">高级：模板复用 (Include)</h3>
						<p>
							可以将标准配置（如 NTP, SNMP, Logging）保存为单独的
							<code v-pre>.txt</code
							>
							文件，然后在主模板中使用 <code v-pre>{% include "..." %}</code
							> 导入它们。
						</p>
						<TemplateBatchCodeSnippet
							class="mt-2"
							:code="snippets.include"
						/>
						<UAlert color="neutral" variant="subtle" class="mt-3">
							<template #description
								>这对于维护全局 ACL、标准安全策略或设备基础配置非常有用，实现“一处修改，处处生效”。</template
							>
						</UAlert>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<h3 class="text-base font-semibold text-green-800">临时变量与格式化</h3>
						<p>可以用 <code v-pre>{% set %}</code> 或过滤器构造动态字符串：</p>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.setBlock" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>format</UKbd><span class="text-sm">格式化输出，如 <code v-pre>{{ vlan.id | format(value="%03d") }}</code></span></div>
							<div class="flex items-center gap-2"><UKbd>trim</UKbd><span class="text-sm text-(--ui-text-muted)">清理空白字符，<code v-pre>{{ desc | trim }}</code></span></div>
						</div>
					</UPageList>
				</section>

				<section>
					<UPageList>
						<header class="space-y-1">
							<h3 class="text-base font-semibold text-green-800">复杂条件与默认值</h3>
							<p>组合 <code>if/elif/else</code> 与 <code>default</code> 可覆盖不同角色。</p>
						</header>
						<TemplateBatchCodeSnippet class="mt-2" :code="snippets.roleCondition" />
						<div class="mt-2 space-y-2">
							<div class="flex items-center gap-2"><UKbd>device.role</UKbd><span class="text-sm">设备角色枚举：<code>core</code>、<code>distribution</code>、<code>access</code></span></div>
							<div class="flex items-center gap-2"><UKbd>default</UKbd><span class="text-sm text-(--ui-text-muted)">结合 fallback 字段避免空值</span></div>
						</div>
					</UPageList>
				</section>

			</div>
		</template>
	</UDrawer>
</template>

<script setup lang="ts">
import { ref } from "vue";

const open = ref(false);
const snippets = {
	variableBasics: `{% set core_name = "core_switch_01" %}
Hostname: {{ core_name }}
{% set vlan_desc = device_site | replace(from=" ", to="_") %}
VLAN10 description: {{ vlan_desc }}
Loopback IP: {{ device.loopback.ip }}`,
	loop: `{# 使用 -%} 来控制循环前后的多余换行 #}
{%- for vlan in vlans -%}
interface Vlan{{ vlan.id }}
  description {{ vlan.name }}
{%- endfor -%}`,
	vlanArray: `[{"id":1,"name":"Core","ip":"10.0.0.1","mask":"255.255.255.0"}]`,
	condition: `{% if device.role == "core" %}
  router ospf 1
{% elif device.role == "distribution" %}
  ip route ...
{% else %}
  interface Vlan99
{% endif %}`,
	portsSingleCard: `{% for port in range(start=1, end=device.port_count + 1) -%}
interface 100GE1/0/{{ port }}
  shutdown
{% endfor %}`,
	portsMultiCard: `{% for slot in range(start=1, end=device.slot_end + 1) %}
{% for port in range(start=0, end=device.port_per_slot) %}
interface 100GE{{ slot }}/0/{{ port }}
 shutdown
{%- endfor %}
{%- endfor %}`,
	defaultSpeed: `speed {{ interface.speed | default(value=interface.max_speed) }}`,
	macro: `{% macro bgp_neighbor(peer, asn, desc) %}
neighbor {{ peer }} remote-as {{ asn }}
neighbor {{ peer }} description {{ desc }}
{% endmacro %}`,
	macroCall: `{{ bgp_neighbor("10.1.1.2", 65001, "Core to PE1") }}`,
	setBlock: `{% set loopback_id = device.id + 100 %}
interface Loopback{{ loopback_id }}
  description {{ device.hostname | replace(from=" ", to="_") | upper }}`,
	roleCondition: `{% if device.role in ["core","distribution"] %}
  router ospf 1
{% elif device.role == "access" %}
  ip route 0.0.0.0 0.0.0.0 {{ device.uplink }}
{% endif %}`,

	// 新增的 snippets
	whitespace: `{# 示例 1: 默认输出 (注意换行) #}
{% for vlan in vlans %}
interface Vlan{{ vlan.id }}
{% endfor %}

{# --- 输出 ---
interface Vlan10
<空行>
interface Vlan20
#}

{# 示例 2: 使用空白控制 (紧凑) #}
{%- for vlan in vlans -%}
interface Vlan{{ vlan.id }}
{%- endfor -%}

{# --- 输出 (没有多余换行) ---
interface Vlan10
interface Vlan20
#}`,
	comments: `{# 这是模板注释，不会输出 #}
{# 遍历所有核心接口 #}
{% for iface in core_interfaces %}
interface {{ iface.name }}
  ! 这是设备注释，会输出到配置里
  description {{ iface.desc }}
{% endfor %}`,
	advCondition: `{# 场景：核心设备，或者是启用了 OSPF 的汇聚设备 #}
{% if device.role == "core" or (device.role == "distribution" and device.ospf_enabled) %}
router ospf 1
  router-id {{ device.loopback.ip }}
{% endif %}

{# F场景：非虚拟设备 #}
{% if not device.is_virtual %}
logging trap errors
{% endif %}`,
	include: `! === 基础配置 ===
hostname {{ device.hostname }}
!
! === 导入标准安全配置 ===
{% include "standard_security_v1.txt" %}
!
! === 导入标准NTP/Logging配置 ===
{% include "standard_mgmt.txt" %}
!
! === 业务配置 ===
interface Vlan100
  ...`,
	math: `{% set vlan_id = device.vlan_base + 10 %}
{% set vlan_id_str = vlan_id | format(value="%d") %}
{% set gw_offset_str = device.gw_offset | format(value="%d") %}
{% set gateway_ip = "10." ~ vlan_id_str ~ ".1." ~ gw_offset_str %}
interface Vlan{{ vlan_id }}
  description Service_Vlan
  ip address {{ gateway_ip }} 255.255.255.0`
};
</script>
