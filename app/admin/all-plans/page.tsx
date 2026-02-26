import { AdminAuthProvider } from "@/context/AdminAuthContext";
import AllPlansPageContent from "./AllPlansPageContent";

export default function AllPlansPage() {
  return (
    <AdminAuthProvider>
      <AllPlansPageContent />
    </AdminAuthProvider>
  );
}