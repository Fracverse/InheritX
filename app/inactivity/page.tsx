"use client";

import { motion } from "framer-motion";
import { Clock, AlertCircle, CheckCircle2, Settings, Bell, Shield } from "lucide-react";
import { useState } from "react";

interface InactivitySetting {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  enabled: boolean;
  days?: number;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
      delayChildren: 0.2,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.5 },
  },
};

export default function InactivityPage() {
  const [settings, setSettings] = useState<InactivitySetting[]>([
    {
      id: "proof-of-life",
      title: "Proof of Life Check-ins",
      description: "Require periodic check-ins to confirm account activity",
      icon: <Clock className="w-6 h-6" />,
      enabled: true,
      days: 30,
    },
    {
      id: "notifications",
      title: "Inactivity Notifications",
      description: "Receive alerts before inactivity triggers are activated",
      icon: <Bell className="w-6 h-6" />,
      enabled: true,
      days: 7,
    },
    {
      id: "auto-transfer",
      title: "Automatic Asset Transfer",
      description: "Automatically transfer assets to beneficiaries after inactivity period",
      icon: <Shield className="w-6 h-6" />,
      enabled: false,
      days: 90,
    },
  ]);

  const [selectedSetting, setSelectedSetting] = useState<string | null>(null);
  const [customDays, setCustomDays] = useState<{ [key: string]: number }>({});

  const toggleSetting = (id: string) => {
    setSettings(
      settings.map((s) =>
        s.id === id ? { ...s, enabled: !s.enabled } : s
      )
    );
  };

  const updateDays = (id: string, days: number) => {
    setSettings(
      settings.map((s) =>
        s.id === id ? { ...s, days } : s
      )
    );
    setCustomDays({ ...customDays, [id]: days });
  };

  const activeSetting = settings.find((s) => s.id === selectedSetting);

  return (
    <div className="min-h-screen bg-linear-to-b from-black via-slate-950 to-black text-white">
      {/* Header */}
      <motion.section
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        className="max-w-7xl mx-auto px-6 pt-20 pb-12"
      >
        <div className="flex items-center gap-3 mb-4">
          <Clock className="w-8 h-8 text-blue-400" />
          <h1 className="text-4xl md:text-5xl font-bold">Account Inactivity</h1>
        </div>
        <p className="text-lg text-slate-300 max-w-2xl">
          Manage your account inactivity settings and ensure your assets are protected
          with automated proof-of-life checks and beneficiary notifications.
        </p>
      </motion.section>

      {/* Main Content */}
      <section className="max-w-7xl mx-auto px-6 pb-20">
        <div className="grid lg:grid-cols-3 gap-8">
          {/* Settings List */}
          <motion.div
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            className="lg:col-span-2 space-y-4"
          >
            <h2 className="text-2xl font-semibold mb-6">Inactivity Settings</h2>
            {settings.map((setting) => (
              <motion.div
                key={setting.id}
                variants={itemVariants}
                onClick={() => setSelectedSetting(setting.id)}
                className={`p-6 rounded-2xl border-2 cursor-pointer transition-all duration-300 ${
                  selectedSetting === setting.id
                    ? "border-blue-500 bg-blue-500/10"
                    : "border-slate-800 bg-slate-900/50 hover:border-slate-700"
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-4 flex-1">
                    <div className="text-blue-400 mt-1">{setting.icon}</div>
                    <div className="flex-1">
                      <h3 className="text-lg font-semibold mb-2">{setting.title}</h3>
                      <p className="text-slate-400 text-sm">{setting.description}</p>
                      {setting.days && (
                        <p className="text-slate-500 text-xs mt-3">
                          Interval: {setting.days} days
                        </p>
                      )}
                    </div>
                  </div>
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.95 }}
                    onClick={(e) => {
                      e.stopPropagation();
                      toggleSetting(setting.id);
                    }}
                    className={`relative w-12 h-7 rounded-full transition-colors duration-300 ${
                      setting.enabled ? "bg-blue-500" : "bg-slate-700"
                    }`}
                  >
                    <motion.div
                      animate={{ x: setting.enabled ? 20 : 2 }}
                      transition={{ type: "spring", stiffness: 500, damping: 30 }}
                      className="absolute top-1 left-1 w-5 h-5 bg-white rounded-full"
                    />
                  </motion.button>
                </div>
              </motion.div>
            ))}
          </motion.div>

          {/* Settings Panel */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.6, delay: 0.2 }}
            className="lg:col-span-1"
          >
            <div className="sticky top-20 rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
              {activeSetting ? (
                <>
                  <div className="flex items-center gap-3 mb-6">
                    <div className="text-blue-400">{activeSetting.icon}</div>
                    <h3 className="text-xl font-semibold">{activeSetting.title}</h3>
                  </div>

                  <div className="space-y-6">
                    {/* Status */}
                    <div>
                      <label className="text-sm font-medium text-slate-300 block mb-3">
                        Status
                      </label>
                      <div
                        className={`flex items-center gap-2 p-3 rounded-lg ${
                          activeSetting.enabled
                            ? "bg-green-500/10 border border-green-500/30"
                            : "bg-slate-800/50 border border-slate-700"
                        }`}
                      >
                        {activeSetting.enabled ? (
                          <>
                            <CheckCircle2 className="w-5 h-5 text-green-400" />
                            <span className="text-sm text-green-300">Enabled</span>
                          </>
                        ) : (
                          <>
                            <AlertCircle className="w-5 h-5 text-slate-400" />
                            <span className="text-sm text-slate-400">Disabled</span>
                          </>
                        )}
                      </div>
                    </div>

                    {/* Days Interval */}
                    {activeSetting.days && (
                      <div>
                        <label className="text-sm font-medium text-slate-300 block mb-3">
                          Check-in Interval (days)
                        </label>
                        <div className="flex items-center gap-3">
                          <input
                            type="range"
                            min="7"
                            max="365"
                            value={customDays[activeSetting.id] || activeSetting.days}
                            onChange={(e) =>
                              updateDays(activeSetting.id, parseInt(e.target.value))
                            }
                            className="flex-1 h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
                          />
                          <span className="text-sm font-semibold text-blue-400 min-w-12">
                            {customDays[activeSetting.id] || activeSetting.days}d
                          </span>
                        </div>
                      </div>
                    )}

                    {/* Action Button */}
                    <motion.button
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      onClick={() => toggleSetting(activeSetting.id)}
                      className={`w-full py-3 rounded-lg font-semibold transition-colors duration-300 ${
                        activeSetting.enabled
                          ? "bg-red-500/20 text-red-300 hover:bg-red-500/30 border border-red-500/30"
                          : "bg-blue-500 text-white hover:bg-blue-600 border border-blue-600"
                      }`}
                    >
                      {activeSetting.enabled ? "Disable" : "Enable"}
                    </motion.button>
                  </div>
                </>
              ) : (
                <div className="text-center py-8">
                  <Settings className="w-12 h-12 text-slate-600 mx-auto mb-3" />
                  <p className="text-slate-400">Select a setting to configure</p>
                </div>
              )}
            </div>
          </motion.div>
        </div>
      </section>

      {/* Info Section */}
      <motion.section
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        viewport={{ once: true }}
        className="max-w-7xl mx-auto px-6 pb-20"
      >
        <div className="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
          <h3 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <Shield className="w-6 h-6 text-blue-400" />
            How Inactivity Protection Works
          </h3>
          <div className="grid md:grid-cols-3 gap-6 mt-6">
            {[
              {
                step: "1",
                title: "Regular Check-ins",
                description:
                  "You'll receive periodic reminders to confirm your account activity",
              },
              {
                step: "2",
                title: "Beneficiary Notification",
                description:
                  "If you miss check-ins, your beneficiaries are notified of potential asset transfer",
              },
              {
                step: "3",
                title: "Automatic Transfer",
                description:
                  "After the inactivity period expires, assets are automatically transferred to beneficiaries",
              },
            ].map((item, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: i * 0.1 }}
                viewport={{ once: true }}
                className="text-center"
              >
                <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-blue-500/20 border border-blue-500/30 mb-4">
                  <span className="font-bold text-blue-400">{item.step}</span>
                </div>
                <h4 className="font-semibold mb-2">{item.title}</h4>
                <p className="text-sm text-slate-400">{item.description}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </motion.section>

      {/* CTA Section */}
      <motion.section
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        viewport={{ once: true }}
        className="max-w-7xl mx-auto px-6 pb-20"
      >
        <div className="rounded-2xl bg-linear-to-r from-blue-600/20 to-purple-600/20 border border-blue-500/30 p-8 text-center">
          <h3 className="text-2xl font-semibold mb-3">Ready to Protect Your Assets?</h3>
          <p className="text-slate-300 mb-6 max-w-2xl mx-auto">
            Configure your inactivity settings now to ensure your inheritance plan is
            secure and your beneficiaries are protected.
          </p>
          <motion.button
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            className="px-8 py-3 rounded-lg bg-blue-500 text-white font-semibold hover:bg-blue-600 transition-colors"
          >
            Save Settings
          </motion.button>
        </div>
      </motion.section>
    </div>
  );
}
