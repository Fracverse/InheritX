"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import * as d3 from "d3";

export type FamilyLayout = "hierarchical" | "radial" | "force";

export interface FamilyMemberNode {
  id: string;
  name: string;
  relationshipLabel: string;
  relationshipDegree: number;
  geneticSimilarity: number;
  verificationStatus: "verified" | "pending" | "partial_match" | "unverified";
  confidenceLevel: number;
  healthRiskScore: number;
  healthConditions: string[];
}

export interface GeneticConnectionEdge {
  id: string;
  sourceId: string;
  targetId: string;
  relationshipType: string;
  relationshipDegree: number;
  similarityStrength: number;
  confidenceLevel: number;
  verified: boolean;
}

export interface InheritanceFlowEdge {
  id: string;
  sourceId: string;
  targetId: string;
  assetType: string;
  amount: number;
  currency: string;
  status: "planned" | "active" | "distributed";
}

export interface InteractiveFamilyTreeProps {
  members: FamilyMemberNode[];
  geneticConnections: GeneticConnectionEdge[];
  inheritanceFlows: InheritanceFlowEdge[];
  width?: number;
  height?: number;
  defaultCenterMemberId?: string;
}

interface PositionedNode extends FamilyMemberNode {
  x: number;
  y: number;
}

const HEALTH_RISK_COLORS = {
  low: "#48BB78",
  medium: "#F6AD55",
  high: "#F56565",
} as const;

function getHealthRiskBand(score: number): keyof typeof HEALTH_RISK_COLORS {
  if (score >= 70) return "high";
  if (score >= 40) return "medium";
  return "low";
}

function getSimilarityColor(similarityStrength: number): string {
  if (similarityStrength >= 80) return "#63B3ED";
  if (similarityStrength >= 60) return "#81E6D9";
  if (similarityStrength >= 40) return "#F6E05E";
  return "#A0AEC0";
}

function buildTreeHierarchy(
  centerId: string,
  members: FamilyMemberNode[],
  connections: GeneticConnectionEdge[],
): d3.HierarchyNode<FamilyMemberNode> {
  const memberById = new Map(members.map((member) => [member.id, member]));
  const childrenById = new Map<string, string[]>();

  for (const connection of connections) {
    if (connection.relationshipType === "parent") {
      const children = childrenById.get(connection.sourceId) ?? [];
      children.push(connection.targetId);
      childrenById.set(connection.sourceId, children);
    } else if (connection.relationshipType === "child") {
      const children = childrenById.get(connection.targetId) ?? [];
      children.push(connection.sourceId);
      childrenById.set(connection.targetId, children);
    }
  }

  const fallbackRoot = members[0];
  const rootMember = memberById.get(centerId) ?? fallbackRoot;
  if (!rootMember) {
    throw new Error("InteractiveFamilyTree requires at least one member");
  }

  interface FamilyHierarchyNode {
    node: FamilyMemberNode;
    children: FamilyHierarchyNode[];
  }

  const visited = new Set<string>();
  const buildNode = (id: string): FamilyHierarchyNode => {
    visited.add(id);
    const member = memberById.get(id);
    if (!member) {
      return {
        node: rootMember,
        children: [],
      };
    }

    const childIds = (childrenById.get(id) ?? []).filter((childId) => !visited.has(childId));
    return {
      node: member,
      children: childIds.map((childId) => buildNode(childId)),
    };
  };

  const hierarchyRoot = buildNode(rootMember.id);
  for (const member of members) {
    if (!visited.has(member.id)) {
      hierarchyRoot.children.push({
        node: member,
        children: [],
      });
    }
  }

  return d3.hierarchy<FamilyHierarchyNode>(hierarchyRoot, (item) => item.children).each((node) => {
    (node as any).data = (node.data as any).node;
  }) as unknown as d3.HierarchyNode<FamilyMemberNode>;
}

function formatCurrency(amount: number, currency: string): string {
  return `${currency} ${amount.toLocaleString()}`;
}

export default function InteractiveFamilyTree({
  members,
  geneticConnections,
  inheritanceFlows,
  width = 980,
  height = 620,
  defaultCenterMemberId,
}: InteractiveFamilyTreeProps) {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const viewportRef = useRef<SVGGElement | null>(null);
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null);

  const [layout, setLayout] = useState<FamilyLayout>("hierarchical");
  const [showGenetics, setShowGenetics] = useState(true);
  const [showInheritance, setShowInheritance] = useState(true);
  const [showHealth, setShowHealth] = useState(true);
  const [centerMemberId, setCenterMemberId] = useState<string>(defaultCenterMemberId ?? "");
  const [selectedMemberId, setSelectedMemberId] = useState<string>("");

  const activeCenterMemberId = useMemo(() => {
    if (members.length === 0) return "";
    if (centerMemberId && members.some((member) => member.id === centerMemberId)) {
      return centerMemberId;
    }
    if (
      defaultCenterMemberId &&
      members.some((member) => member.id === defaultCenterMemberId)
    ) {
      return defaultCenterMemberId;
    }
    return members[0].id;
  }, [centerMemberId, defaultCenterMemberId, members]);

  const activeSelectedMemberId = useMemo(() => {
    if (members.length === 0) return "";
    if (selectedMemberId && members.some((member) => member.id === selectedMemberId)) {
      return selectedMemberId;
    }
    return members[0].id;
  }, [members, selectedMemberId]);

  const filteredGraph = useMemo(() => {
    if (!activeCenterMemberId) {
      return { members: [], geneticConnections: [], inheritanceFlows: [] };
    }

    const adjacency = new Map<string, string[]>();
    const connect = (sourceId: string, targetId: string) => {
      const current = adjacency.get(sourceId) ?? [];
      current.push(targetId);
      adjacency.set(sourceId, current);
    };

    for (const edge of geneticConnections) {
      connect(edge.sourceId, edge.targetId);
      connect(edge.targetId, edge.sourceId);
    }
    for (const edge of inheritanceFlows) {
      connect(edge.sourceId, edge.targetId);
      connect(edge.targetId, edge.sourceId);
    }

    const depthLimit = 2;
    const distanceByMember = new Map<string, number>([[activeCenterMemberId, 0]]);
    const queue = [activeCenterMemberId];
    while (queue.length > 0) {
      const current = queue.shift()!;
      const distance = distanceByMember.get(current) ?? 0;
      if (distance >= depthLimit) continue;
      for (const nextId of adjacency.get(current) ?? []) {
        if (!distanceByMember.has(nextId)) {
          distanceByMember.set(nextId, distance + 1);
          queue.push(nextId);
        }
      }
    }

    const visibleMemberIds = new Set(distanceByMember.keys());
    const scopedMembers = members.filter((member) => visibleMemberIds.has(member.id));
    const scopedGenetics = geneticConnections.filter(
      (connection) =>
        visibleMemberIds.has(connection.sourceId) &&
        visibleMemberIds.has(connection.targetId),
    );
    const scopedFlows = inheritanceFlows.filter(
      (flow) => visibleMemberIds.has(flow.sourceId) && visibleMemberIds.has(flow.targetId),
    );

    return {
      members: scopedMembers,
      geneticConnections: scopedGenetics,
      inheritanceFlows: scopedFlows,
    };
  }, [activeCenterMemberId, members, geneticConnections, inheritanceFlows]);

  const positionedNodes = useMemo(() => {
    const activeMembers = filteredGraph.members;
    const activeConnections = filteredGraph.geneticConnections;
    if (activeMembers.length === 0) return [] as PositionedNode[];

    if (layout === "force") {
      const nodes = activeMembers.map((member) => ({ ...member })) as Array<
        FamilyMemberNode & d3.SimulationNodeDatum
      >;
      const links = activeConnections.map((connection) => ({
        source: connection.sourceId,
        target: connection.targetId,
      }));

      const simulation = d3
        .forceSimulation(nodes)
        .force(
          "link",
          d3
            .forceLink(links)
            .id((d: any) => d.id)
            .distance(140),
        )
        .force("charge", d3.forceManyBody().strength(-320))
        .force("center", d3.forceCenter(width / 2, height / 2))
        .stop();

      for (let i = 0; i < 220; i += 1) {
        simulation.tick();
      }

      return nodes.map((node) => ({
        ...node,
        x: node.x ?? width / 2,
        y: node.y ?? height / 2,
      }));
    }

    const hierarchy = buildTreeHierarchy(
      activeCenterMemberId,
      activeMembers,
      activeConnections,
    );
    if (layout === "radial") {
      const radialTree = d3.tree<FamilyMemberNode>().size([2 * Math.PI, Math.min(width, height) / 2.6]);
      const root = radialTree(hierarchy);
      const centerX = width / 2;
      const centerY = height / 2;
      return root.descendants().map((node) => ({
        ...node.data,
        x: centerX + node.y * Math.cos(node.x - Math.PI / 2),
        y: centerY + node.y * Math.sin(node.x - Math.PI / 2),
      }));
    }

    const tree = d3.tree<FamilyMemberNode>().size([height - 140, width - 200]);
    const root = tree(hierarchy);
    return root.descendants().map((node) => ({
      ...node.data,
      x: node.y + 100,
      y: node.x + 70,
    }));
  }, [
    activeCenterMemberId,
    filteredGraph.geneticConnections,
    filteredGraph.members,
    height,
    layout,
    width,
  ]);

  const positionedById = useMemo(() => {
    return new Map(positionedNodes.map((node) => [node.id, node]));
  }, [positionedNodes]);

  const selectedMember = useMemo(() => {
    return (
      filteredGraph.members.find((member) => member.id === activeSelectedMemberId) ??
      null
    );
  }, [activeSelectedMemberId, filteredGraph.members]);

  const selectedMemberConnections = useMemo(() => {
    if (!activeSelectedMemberId) return [];
    return filteredGraph.geneticConnections.filter(
      (connection) =>
        connection.sourceId === activeSelectedMemberId ||
        connection.targetId === activeSelectedMemberId,
    );
  }, [activeSelectedMemberId, filteredGraph.geneticConnections]);

  const selectedMemberFlows = useMemo(() => {
    if (!activeSelectedMemberId) return [];
    return filteredGraph.inheritanceFlows.filter(
      (flow) =>
        flow.sourceId === activeSelectedMemberId ||
        flow.targetId === activeSelectedMemberId,
    );
  }, [activeSelectedMemberId, filteredGraph.inheritanceFlows]);

  useEffect(() => {
    if (!svgRef.current || !viewportRef.current) return;

    const svg = d3.select(svgRef.current);
    const viewport = d3.select(viewportRef.current);
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.35, 4])
      .on("zoom", (event) => {
        viewport.attr("transform", event.transform.toString());
      });

    zoomRef.current = zoom;
    svg.call(zoom);
    svg.call(zoom.transform, d3.zoomIdentity);

    return () => {
      svg.on(".zoom", null);
    };
  }, []);

  const handleZoom = (factor: number) => {
    if (!svgRef.current || !zoomRef.current) return;
    d3.select(svgRef.current).call(zoomRef.current.scaleBy, factor);
  };

  const handleResetView = () => {
    if (!svgRef.current || !zoomRef.current) return;
    d3.select(svgRef.current).call(zoomRef.current.transform, d3.zoomIdentity);
  };

  if (members.length === 0) {
    return (
      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
        <p className="text-sm text-[#92A5A8]">No family members available for tree rendering.</p>
      </div>
    );
  }

  return (
    <section className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-xl font-semibold text-white">Interactive Family Tree</h2>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => handleZoom(1.2)}
            className="rounded-md border border-[#1C252A] px-3 py-1.5 text-xs text-[#92A5A8] hover:bg-[#1C252A] hover:text-white"
            aria-label="Zoom in"
          >
            Zoom In
          </button>
          <button
            type="button"
            onClick={() => handleZoom(0.8)}
            className="rounded-md border border-[#1C252A] px-3 py-1.5 text-xs text-[#92A5A8] hover:bg-[#1C252A] hover:text-white"
            aria-label="Zoom out"
          >
            Zoom Out
          </button>
          <button
            type="button"
            onClick={handleResetView}
            className="rounded-md border border-[#1C252A] px-3 py-1.5 text-xs text-[#92A5A8] hover:bg-[#1C252A] hover:text-white"
            aria-label="Reset view"
          >
            Reset View
          </button>
        </div>
      </div>

      <div className="mb-5 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
        <label className="flex flex-col gap-1 text-xs text-[#92A5A8]">
          Layout algorithm
          <select
            value={layout}
            onChange={(event) => setLayout(event.target.value as FamilyLayout)}
            className="rounded-md border border-[#1C252A] bg-[#0A0F11] px-3 py-2 text-sm text-white outline-none focus:border-[#33C5E0]"
            aria-label="Layout algorithm"
          >
            <option value="hierarchical">Hierarchical</option>
            <option value="radial">Radial</option>
            <option value="force">Force-directed</option>
          </select>
        </label>
        <label className="flex flex-col gap-1 text-xs text-[#92A5A8]">
          Center member
          <select
            value={activeCenterMemberId}
            onChange={(event) => setCenterMemberId(event.target.value)}
            className="rounded-md border border-[#1C252A] bg-[#0A0F11] px-3 py-2 text-sm text-white outline-none focus:border-[#33C5E0]"
            aria-label="Center member"
          >
            {members.map((member) => (
              <option key={member.id} value={member.id}>
                {member.name}
              </option>
            ))}
          </select>
        </label>

        <label className="flex items-center gap-2 rounded-md border border-[#1C252A] bg-[#0A0F11] px-3 py-2 text-sm text-[#D9E4E6]">
          <input
            type="checkbox"
            className="h-4 w-4 accent-[#33C5E0]"
            checked={showGenetics}
            onChange={(event) => setShowGenetics(event.target.checked)}
            aria-label="Toggle genetic links"
          />
          Genetic links
        </label>
        <label className="flex items-center gap-2 rounded-md border border-[#1C252A] bg-[#0A0F11] px-3 py-2 text-sm text-[#D9E4E6]">
          <input
            type="checkbox"
            className="h-4 w-4 accent-[#33C5E0]"
            checked={showInheritance}
            onChange={(event) => setShowInheritance(event.target.checked)}
            aria-label="Toggle inheritance flows"
          />
          Inheritance flows
        </label>
        <label className="flex items-center gap-2 rounded-md border border-[#1C252A] bg-[#0A0F11] px-3 py-2 text-sm text-[#D9E4E6]">
          <input
            type="checkbox"
            className="h-4 w-4 accent-[#33C5E0]"
            checked={showHealth}
            onChange={(event) => setShowHealth(event.target.checked)}
            aria-label="Toggle health indicators"
          />
          Health indicators
        </label>
      </div>

      <div className="grid grid-cols-1 gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
        <div className="rounded-xl border border-[#1C252A] bg-[#0A0F11] p-2">
          <svg
            ref={svgRef}
            width="100%"
            height={height}
            viewBox={`0 0 ${width} ${height}`}
            className="w-full cursor-grab rounded-lg active:cursor-grabbing"
            role="img"
            aria-label="Family tree graph"
          >
            <defs>
              <marker
                id="asset-arrow"
                markerWidth="8"
                markerHeight="8"
                refX="7"
                refY="4"
                orient="auto"
              >
                <path d="M0,0 L8,4 L0,8 Z" fill="#F6AD55" />
              </marker>
            </defs>

            <g ref={viewportRef}>
              {showGenetics &&
                filteredGraph.geneticConnections.map((connection) => {
                  const source = positionedById.get(connection.sourceId);
                  const target = positionedById.get(connection.targetId);
                  if (!source || !target) return null;

                  const strokeColor = getSimilarityColor(connection.similarityStrength);
                  const strokeWidth = Math.max(1.2, connection.confidenceLevel / 38);
                  const labelX = (source.x + target.x) / 2;
                  const labelY = (source.y + target.y) / 2;

                  return (
                    <g key={connection.id}>
                      <line
                        x1={source.x}
                        y1={source.y}
                        x2={target.x}
                        y2={target.y}
                        stroke={strokeColor}
                        strokeWidth={strokeWidth}
                        strokeOpacity={connection.verified ? 0.85 : 0.55}
                      />
                      <text
                        x={labelX}
                        y={labelY - 7}
                        textAnchor="middle"
                        className="fill-[#A8BBC0] text-[10px]"
                      >
                        {`${connection.relationshipType} • d${connection.relationshipDegree} • ${connection.confidenceLevel}%`}
                      </text>
                    </g>
                  );
                })}

              {showInheritance &&
                filteredGraph.inheritanceFlows.map((flow) => {
                  const source = positionedById.get(flow.sourceId);
                  const target = positionedById.get(flow.targetId);
                  if (!source || !target) return null;

                  const path = d3.path();
                  const midX = (source.x + target.x) / 2;
                  const curveHeight = Math.min(70, Math.abs(source.x - target.x) / 3 + 20);
                  path.moveTo(source.x, source.y);
                  path.quadraticCurveTo(midX, Math.min(source.y, target.y) - curveHeight, target.x, target.y);
                  const pathData = path.toString();

                  return (
                    <g key={flow.id}>
                      <path
                        d={pathData}
                        fill="none"
                        stroke="#F6AD55"
                        strokeWidth={2}
                        strokeDasharray="7 5"
                        markerEnd="url(#asset-arrow)"
                        className="asset-flow-line"
                      />
                      <text
                        x={midX}
                        y={Math.min(source.y, target.y) - curveHeight - 6}
                        textAnchor="middle"
                        className="fill-[#F6C47A] text-[10px]"
                      >
                        {`${flow.assetType}: ${formatCurrency(flow.amount, flow.currency)}`}
                      </text>
                    </g>
                  );
                })}

              {positionedNodes.map((node) => {
                const riskBand = getHealthRiskBand(node.healthRiskScore);
                const riskColor = HEALTH_RISK_COLORS[riskBand];
                const isSelected = node.id === activeSelectedMemberId;
                const initials = node.name
                  .split(" ")
                  .map((part) => part[0] ?? "")
                  .join("")
                  .slice(0, 2)
                  .toUpperCase();

                return (
                  <g
                    key={node.id}
                    transform={`translate(${node.x}, ${node.y})`}
                    onClick={() => setSelectedMemberId(node.id)}
                    role="button"
                    aria-label={`Member ${node.name}`}
                    tabIndex={0}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        setSelectedMemberId(node.id);
                      }
                    }}
                    className="cursor-pointer"
                    data-testid={`member-node-${node.id}`}
                  >
                    {showHealth && (
                      <circle
                        r={23}
                        fill="none"
                        stroke={riskColor}
                        strokeOpacity={0.8}
                        strokeWidth={2.3}
                      />
                    )}
                    <circle
                      r={17}
                      fill={isSelected ? "#33C5E0" : "#1C252A"}
                      stroke={node.verificationStatus === "verified" ? "#48BB78" : "#718096"}
                      strokeWidth={2}
                    />
                    <text
                      textAnchor="middle"
                      dy="0.33em"
                      className="pointer-events-none fill-white text-[10px] font-semibold"
                    >
                      {initials}
                    </text>
                    <text x={0} y={34} textAnchor="middle" className="fill-[#D9E4E6] text-[10px]">
                      {node.name}
                    </text>
                  </g>
                );
              })}
            </g>
          </svg>
        </div>

        <aside className="rounded-xl border border-[#1C252A] bg-[#0A0F11] p-4">
          <h3 className="text-base font-semibold text-white">Member Details</h3>
          {!selectedMember ? (
            <p className="mt-3 text-sm text-[#92A5A8]">Select a family member to inspect details.</p>
          ) : (
            <div className="mt-3 space-y-3 text-sm">
              <div className="rounded-lg border border-[#1C252A] bg-[#11191D] p-3">
                <p className="text-xs text-[#6B7C7F]">Name</p>
                <p className="mt-1 font-medium text-white">{selectedMember.name}</p>
                <p className="mt-1 text-xs text-[#92A5A8]">{selectedMember.relationshipLabel}</p>
              </div>

              <div className="rounded-lg border border-[#1C252A] bg-[#11191D] p-3">
                <p className="text-xs text-[#6B7C7F]">Genetic profile</p>
                <p className="mt-1 text-white">
                  Similarity {selectedMember.geneticSimilarity}% | Confidence{" "}
                  {selectedMember.confidenceLevel}%
                </p>
                <p className="mt-1 text-[#92A5A8]">
                  Verification: {selectedMember.verificationStatus.replace("_", " ")}
                </p>
              </div>

              <div className="rounded-lg border border-[#1C252A] bg-[#11191D] p-3">
                <p className="text-xs text-[#6B7C7F]">Health risk</p>
                <p className="mt-1 text-white">Risk score: {selectedMember.healthRiskScore}/100</p>
                <p className="mt-1 text-[#92A5A8]">
                  {selectedMember.healthConditions.length > 0
                    ? selectedMember.healthConditions.join(", ")
                    : "No critical conditions flagged"}
                </p>
              </div>

              <div className="rounded-lg border border-[#1C252A] bg-[#11191D] p-3">
                <p className="text-xs text-[#6B7C7F]">Connections</p>
                <p className="mt-1 text-white">{selectedMemberConnections.length} genetic connections</p>
                <p className="mt-1 text-white">{selectedMemberFlows.length} inheritance flows</p>
              </div>
            </div>
          )}
        </aside>
      </div>

      <style>{`
        .asset-flow-line {
          animation: flow-dash 2.6s linear infinite;
        }

        @keyframes flow-dash {
          to {
            stroke-dashoffset: -24;
          }
        }
      `}</style>
    </section>
  );
}
