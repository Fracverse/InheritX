"use client";

import React, { useState } from "react";
import {
  ArrowDownRight,
  ArrowUpRight,
  BadgeCheck,
  CheckIcon,
  LogOut,
} from "lucide-react";
import Toggle from "./components/Toggle";
import EnableTwoFactorModal from "./components/EnableTwoFactorModal";
import TwoFactorSuccessModal from "./components/TwoFactorSuccessModal";
import EnableFingerprintModal from "./components/EnableFingerprintModal";
import EnableFaceIdModal from "./components/EnableFaceIdModal";

interface Device {
  type: "DESKTOP" | "MOBILE";
  name: string;
}

export default function SecurityPage() {
  // Toggle states
  const [twoFactorEnabled, setTwoFactorEnabled] = useState(false);
  const [fingerprintEnabled, setFingerprintEnabled] = useState(false);
  const [faceIdEnabled, setFaceIdEnabled] = useState(false);
  const [emailAlertsEnabled, setEmailAlertsEnabled] = useState(false);

  // Modal states
  const [showTwoFactorModal, setShowTwoFactorModal] = useState(false);
  const [showTwoFactorSuccess, setShowTwoFactorSuccess] = useState(false);
  const [showFingerprintModal, setShowFingerprintModal] = useState(false);
  const [showFaceIdModal, setShowFaceIdModal] = useState(false);

  // Devices
  const [devices] = useState<Device[]>([
    { type: "DESKTOP", name: "Mac M1" },
    { type: "MOBILE", name: "Samsung S21 FE" },
  ]);

  const handleTwoFactorToggle = (enabled: boolean) => {
    if (enabled && !twoFactorEnabled) {
      // Opening modal to enable
      setShowTwoFactorModal(true);
    } else if (!enabled && twoFactorEnabled) {
      // Directly disable if already enabled
      setTwoFactorEnabled(false);
    }
  };

  const handleTwoFactorSuccess = () => {
    setShowTwoFactorModal(false);
    setShowTwoFactorSuccess(true);
    setTwoFactorEnabled(true);
  };

  const handleTwoFactorSuccessContinue = () => {
    setShowTwoFactorSuccess(false);
  };

  const handleFingerprintToggle = (enabled: boolean) => {
    if (enabled && !fingerprintEnabled) {
      setShowFingerprintModal(true);
    } else if (!enabled && fingerprintEnabled) {
      setFingerprintEnabled(false);
    }
  };

  const handleFingerprintContinue = () => {
    setShowFingerprintModal(false);
    setFingerprintEnabled(true);
  };

  const handleFaceIdToggle = (enabled: boolean) => {
    if (enabled && !faceIdEnabled) {
      setShowFaceIdModal(true);
    } else if (!enabled && faceIdEnabled) {
      setFaceIdEnabled(false);
    }
  };

  const handleFaceIdContinue = () => {
    setShowFaceIdModal(false);
    setFaceIdEnabled(true);
  };

  const handleEmailAlertsToggle = (enabled: boolean) => {
    // No modal, just toggle
    setEmailAlertsEnabled(enabled);
  };

  const handleLogoutDevice = (deviceIndex: number) => {
    // Handle device logout logic here
    console.log("Logging out device:", devices[deviceIndex]);
    // In a real app, you would make an API call here
  };

  const handleSaveSettings = () => {
    // Handle save settings logic here
    console.log("Saving settings...", {
      twoFactorEnabled,
      fingerprintEnabled,
      faceIdEnabled,
      emailAlertsEnabled,
    });
    // In a real app, you would make an API call here
  };

  // Determine account status
  const isAccountSecured = twoFactorEnabled;

  return (
    <div className="space-y-8 max-w-[796px] ">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-4">
        <div>
          <h1 className="text-3xl md:text-4xl font-semibold text-[#FCFFFF] mb-2">
            Security
          </h1>
          <p className="text-sm md:text-base text-[#92A5A8]">
            Manage how you protect your account
          </p>
        </div>

        {/* Account Status */}
        {isAccountSecured ? (
          <div className="flex flex-col items-end gap-2">
            <div className="flex items-center gap-2 bg-green-500/20 border border-green-500/30 rounded-full px-4 py-2">
              <BadgeCheck className="w-[24px] h-[24px] outline-transparent text-[#1C252A] fill-green-500" />
              <span className="text-sm font-medium text-green-400">
                ACCOUNT IS SECURED
              </span>
            </div>
            <p className="text-xs text-[#92A5A8]">
              Your account is Secured. 2FA is activated
            </p>
          </div>
        ) : (
          <div className="flex flex-col items-end gap-2">
            <div className="flex items-center gap-2 bg-red-500/20 border border-[#4D0814] rounded-full px-4 py-2">
              <BadgeCheck className="w-[24px] h-[24px] outline-transparent text-[#1C252A] fill-red-500" />
              <span className="text-sm font-medium text-red-400">
                ACCOUNT NOT VERIFIED
              </span>
            </div>
            <p className="text-xs text-red-400">Enable 2FA</p>
          </div>
        )}
      </div>

      {/* Two-Factor Authentication Section */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-[#FCFFFF]">
          Two-Factor Authentication
        </h2>
        <div className="flex items-center justify-between p-4 bg-[#182024] rounded-xl border border-[#1C252A]">
          <span className="text-[#FCFFFF]">
            Enable Two-Factor Authentication
          </span>
          <Toggle enabled={twoFactorEnabled} onChange={handleTwoFactorToggle} />
        </div>
      </section>

      {/* Device Management Section */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold text-[#FCFFFF]">
            Device Management
          </h2>
          <p className="text-sm text-[#92A5A8]">
            You are logged in on {devices.length} devices
          </p>
        </div>
        <div className="space-y-3">
          {devices.map((device, index) => (
            <div
              key={index}
              className="flex items-center justify-between p-4 bg-[#182024] rounded-xl border border-[#1C252A]"
            >
              <div className="flex flex-col gap-1">
                <span className="text-sm font-medium text-[#92A5A8] uppercase">
                  {device.type}
                </span>
                <span className="text-[#FCFFFF]">{device.name}</span>
              </div>
              <button
                onClick={() => handleLogoutDevice(index)}
                className="flex items-center gap-2 px-4 py-2 border border-[#2A3338] text-[#FCFFFF] rounded-lg hover:bg-[#1C252A] transition-colors text-sm"
              >
                <LogOut size={16} />
                LOG OUT
              </button>
            </div>
          ))}
        </div>
      </section>

      {/* Biometric Authentication Section */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-[#FCFFFF]">
          Biometric Authentication
        </h2>
        <div className="space-y-3">
          <div className="flex items-center justify-between p-4 bg-[#182024] rounded-xl border border-[#1C252A]">
            <span className="text-[#FCFFFF]">Enable Fingerprint</span>
            <Toggle
              enabled={fingerprintEnabled}
              onChange={handleFingerprintToggle}
            />
          </div>
          <div className="flex items-center justify-between p-4 bg-[#182024] rounded-xl border border-[#1C252A]">
            <span className="text-[#FCFFFF]">Enable Face ID</span>
            <Toggle enabled={faceIdEnabled} onChange={handleFaceIdToggle} />
          </div>
        </div>
      </section>

      {/* Security Alerts Section */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold text-[#FCFFFF]">
            Security Alerts
          </h2>
          <p className="text-sm text-[#92A5A8]">
            Get notified of suspicious activity
          </p>
        </div>
        <div className="flex items-center justify-between p-4 bg-[#182024] rounded-xl border border-[#1C252A]">
          <span className="text-[#FCFFFF]">Notify by email</span>
          <Toggle
            enabled={emailAlertsEnabled}
            onChange={handleEmailAlertsToggle}
          />
        </div>
      </section>

      {/* Save Settings Button */}
      <div className="pt-4">
        <button
          onClick={handleSaveSettings}
          className="w-full md:w-auto px-8 py-4 border border-none rounded-b-[32px] rounded-t-8 bg-[#1C252A] text-[#33C5E0] rounded-xl hover:bg-[#1C252A] transition-colors flex items-center justify-center gap-2 font-medium"
        >
          SAVE SETTINGS
          <ArrowDownRight size={20} />
        </button>
      </div>

      {/* Modals */}
      {showTwoFactorModal && (
        <EnableTwoFactorModal
          onClose={() => setShowTwoFactorModal(false)}
          onSuccess={handleTwoFactorSuccess}
        />
      )}

      {showTwoFactorSuccess && (
        <TwoFactorSuccessModal onContinue={handleTwoFactorSuccessContinue} />
      )}

      {showFingerprintModal && (
        <EnableFingerprintModal
          onClose={() => setShowFingerprintModal(false)}
          onContinue={handleFingerprintContinue}
        />
      )}

      {showFaceIdModal && (
        <EnableFaceIdModal
          onClose={() => setShowFaceIdModal(false)}
          onContinue={handleFaceIdContinue}
        />
      )}
    </div>
  );
}
