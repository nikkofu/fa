const state = {
  overview: null,
  selectedTaskId: null,
  selectedTask: null,
  selectedTaskEvidence: [],
  selectedTaskGovernance: null,
  selectedTaskAudit: [],
  toastTimer: null,
  filters: {
    followUp: "all",
    handoff: "all",
    alert: "all",
  },
  autopilotLevel: 2,
  viewMode: "boardroom",
};

const actors = {
  safety_officer: {
    id: "worker_2001",
    display_name: "Wang Safety",
    role: "Safety Officer",
  },
  plant_manager: {
    id: "worker_9001",
    display_name: "Taylor Plant",
    role: "Plant Manager",
  },
  operations_supervisor: {
    id: "worker_1002",
    display_name: "Morgan Ops",
    role: "Operations Supervisor",
  },
  production_supervisor: {
    id: "worker_1001",
    display_name: "Liu Supervisor",
    role: "Production Supervisor",
  },
  maintenance_technician: {
    id: "worker_3001",
    display_name: "Wu Maint",
    role: "Maintenance Technician",
  },
  maintenance_engineer: {
    id: "worker_3201",
    display_name: "Avery Reliability",
    role: "Maintenance Engineer",
  },
  incoming_shift_supervisor: {
    id: "worker_1101",
    display_name: "Zhang Incoming",
    role: "Incoming Shift Supervisor",
  },
  quality_engineer: {
    id: "worker_2101",
    display_name: "Chen QE",
    role: "Quality Engineer",
  },
};

const scenarioBuilders = {
  maintenance_diagnostic(id) {
    return {
      id,
      title: "调查主轴温度漂移",
      description:
        "在下个班次前诊断反复出现的主轴温度漂移，并推荐安全的恢复路径。",
      priority: "critical",
      risk: "high",
      initiator: actors.production_supervisor,
      stakeholders: [],
      equipment_ids: ["eq_cnc_01"],
      integrations: ["mes", "cmms"],
      desired_outcome: "在容差范围内恢复稳定的主轴温度",
      requires_human_approval: true,
      requires_diagnostic_loop: true,
    };
  },
  shift_handoff(id) {
    return {
      id,
      title: "为接班主管总结交接班记录",
      description:
        "在下个班次开始前，提取未解决的问题、受阻的工作和启动风险。",
      priority: "routine",
      risk: "low",
      initiator: actors.production_supervisor,
      stakeholders: [],
      equipment_ids: [],
      integrations: ["mes"],
      desired_outcome: "发布一份清晰的交接班摘要及待办事项",
      requires_human_approval: false,
      requires_diagnostic_loop: false,
    };
  },
  alert_triage(id) {
    return {
      id,
      title: "对包装线 4 上反复出现的安灯警报进行分流",
      description:
        "在升级前审查反复出现的警报爆发并对相似信号进行聚类。",
      priority: "expedited",
      risk: "high",
      initiator: actors.production_supervisor,
      stakeholders: [],
      equipment_ids: ["eq_pack_04"],
      integrations: ["mes"],
      desired_outcome:
        "创建准备好分流的警报集群并将其路由给生产主管。",
      requires_human_approval: false,
      requires_diagnostic_loop: false,
    };
  },
    scada_threshold(id) {
    return {
      id,
      title: "对混合线 2 上持续的温度警报进行分流",
      description:
        "在升级前审查混合线 2 上持续的 SCADA 阈值违规和传感器漂移。",
      priority: "expedited",
      risk: "medium",
      initiator: actors.production_supervisor,
      stakeholders: [],
      equipment_ids: ["eq_mix_02"],
      integrations: ["scada"],
      desired_outcome:
        "聚类持续的阈值信号，并将首次诊断审查路由给维护部门。",
      requires_human_approval: false,
      requires_diagnostic_loop: false,
    };
  },
  margin_arbitrage(id) {
    return {
      id,
      title: "预测性废品套利与动态降级",
      description:
        "检测到产线C设备轻微磨损导致高级SKU良率下降，AI代理建议自动将生产批次降级为标准SKU，并实时在现货市场出售。",
      priority: "critical",
      risk: "high",
      initiator: actors.plant_manager,
      stakeholders: [],
      equipment_ids: ["eq_line_c"],
      integrations: ["mes", "erp"],
      desired_outcome:
        "避免 100% 废品损失，实现利润套利最大化。",
      requires_human_approval: true,
      requires_diagnostic_loop: true,
    };
  },
  capital_allocation(id) {
    return {
      id,
      title: "营运资金动态优化与供应商博弈",
      description:
        "预测到下周订单波动，AI代理自动与 3 家原材料供应商发起实时谈判，延迟交货并释放营运资金。",
      priority: "critical",
      risk: "medium",
      initiator: actors.plant_manager,
      stakeholders: [],
      equipment_ids: [],
      integrations: ["erp", "scada"],
      desired_outcome:
        "优化自由现金流，降低不必要的库存持有成本。",
      requires_human_approval: true,
      requires_diagnostic_loop: false,
    };
  },
  energy_hedging(id) {
    return {
      id,
      title: "能源现货对冲与高耗能产线调度",
      description:
        "电网现货价格飙升，AI代理建议暂停高耗能产线A，转而运行低耗能产线B，并向电网出售闲置电力配额。",
      priority: "expedited",
      risk: "high",
      initiator: actors.plant_manager,
      stakeholders: [],
      equipment_ids: ["eq_line_a", "eq_line_b"],
      integrations: ["scada", "erp"],
      desired_outcome:
        "实现能源成本套利，对冲电力价格波动风险。",
      requires_human_approval: true,
      requires_diagnostic_loop: true,
    };
  },);

  if (!response.ok) {
    const errorText = await readError(response);
    showToast(errorText);
    throw new Error(errorText);
  }

  return response.json();
}

async function postJson(url, body, correlationId) {
  const response = await fetch(url, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      "x-correlation-id": correlationId,
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const errorText = await readError(response);
    showToast(errorText);
    throw new Error(errorText);
  }

  return response.json();
}

async function readError(response) {
  try {
    const payload = await response.json();
    return payload.error || `Request failed with status ${response.status}`;
  } catch (_error) {
    return `Request failed with status ${response.status}`;
  }
}

function showToast(message) {
  if (!elements.toast) {
    return;
  }

  elements.toast.textContent = message;
  elements.toast.classList.add("visible");

  if (state.toastTimer) {
    clearTimeout(state.toastTimer);
  }

  state.toastTimer = setTimeout(() => {
    elements.toast.classList.remove("visible");
  }, 3200);
}

function emptyState(message) {
  return `<div class="empty-state">${escapeHtml(message)}</div>`;
}

function formatNumber(value) {
  return new Intl.NumberFormat("en-US").format(value || 0);
}

function formatDate(value) {
  if (!value) {
    return "Not scheduled";
  }

  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

function toneForStatus(value) {
  const input = String(value || "").toLowerCase();
  if (
    input.includes("critical") ||
    input.includes("high") ||
    input.includes("escalat") ||
    input.includes("failed") ||
    input.includes("overdue")
  ) {
    return "rose";
  }
  if (
    input.includes("medium") ||
    input.includes("warning") ||
    input.includes("awaiting") ||
    input.includes("published")
  ) {
    return "amber";
  }
  if (
    input.includes("accepted") ||
    input.includes("approved") ||
    input.includes("complete") ||
    input.includes("acknowledged")
  ) {
    return "mint";
  }
  return "cyan";
}

function humanize(value) {
  return String(value || "")
    .replace(/_/g, " ")
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function shortId(value) {
  return String(value || "").slice(0, 8);
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
