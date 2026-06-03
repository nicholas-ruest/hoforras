// ADR-0016 — Recovery Planner panel tests: device siting, investment, savings, carbon, advisory.

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  AdvisoryBadge,
  CarbonView,
  InvestmentView,
  SavingsView,
  SitingPlanView,
} from "../components/recovery/RecoveryPanels";
import { demoRecovery } from "../recovery";

const { report } = demoRecovery();

describe("Recovery Planner panels (ADR-0016)", () => {
  it("siting plan shows coverage and proposed device placements (FR-10.2)", () => {
    render(<SitingPlanView report={report} />);
    expect(screen.getByLabelText("siting plan")).toBeInTheDocument();
    expect(screen.getByText(/99%/)).toBeInTheDocument(); // coverage
    expect(screen.getByTestId("observed-loss")).toHaveTextContent("kWh/yr");
  });

  it("investment shows CapEx, OpEx and payback (FR-10.3)", () => {
    render(<InvestmentView report={report} />);
    expect(screen.getByTestId("capex")).toHaveTextContent("€36,060");
    expect(screen.getByTestId("payback")).toHaveTextContent("4.1 yr payback");
  });

  it("savings shows recoverable energy and money saved (FR-10.4)", () => {
    render(<SavingsView report={report} />);
    expect(screen.getByTestId("recoverable")).toHaveTextContent("74,100");
    expect(screen.getByTestId("savings")).toHaveTextContent("€8,892");
  });

  it("carbon shows tonnes CO₂e avoided + relatable equivalents (FR-10.5)", () => {
    render(<CarbonView report={report} />);
    expect(screen.getByTestId("carbon")).toHaveTextContent("17.0");
    expect(screen.getByLabelText("carbon")).toHaveTextContent("trees/yr");
  });

  it("an advisory · estimates-only badge is always present (ADR-0016 §4)", () => {
    render(<AdvisoryBadge />);
    expect(screen.getByTestId("advisory-badge")).toHaveTextContent(/advisory/i);
  });

  it("demo recovery report is internally consistent (savings = recoverable × price)", () => {
    const s = report.savings;
    expect(Math.abs(s.energy_cost_savings_yr - s.recoverable_kwh_yr * s.price_per_kwh)).toBeLessThan(1);
    expect(report.carbon.co2e_kg_yr).toBeCloseTo(
      s.recoverable_kwh_yr * report.carbon.grid_emission_factor_kg_per_kwh,
      0,
    );
  });
});
