import { Link } from "@/components/catalyst-ui/link";
import { useRouter } from "next/router";

export default function Page() {
  const router = useRouter();
  return (
    <div>
      <p>Post: {router.query.id}</p>
      <Link href="/">Back</Link>
    </div>
  );
}
