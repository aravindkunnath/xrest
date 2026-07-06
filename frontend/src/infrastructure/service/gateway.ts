import { ServiceGateway as WailsServiceGateway } from "../../../bindings/xrest/cmd/wails";
import type { IServiceGateway } from "@/domains/service/ports";
import type { Service } from "@/types";

export class ServiceGateway implements IServiceGateway {
  async loadServices(): Promise<Service[]> {
    const result = await WailsServiceGateway.LoadServices();
    return (result as unknown as Service[]) || [];
  }

  async saveServices(
    services: Service[],
    commitMessage?: string,
  ): Promise<Service[]> {
    const result = await WailsServiceGateway.SaveServices(
      services as any,
      commitMessage || "",
    );
    return (result as unknown as Service[]) || [];
  }

  async getGitStatus(directory: string): Promise<any> {
    return WailsServiceGateway.GetGitStatus(directory);
  }

  async initGit(directory: string, remoteUrl?: string): Promise<void> {
    return WailsServiceGateway.InitGit(directory, remoteUrl || "");
  }

  async syncGit(directory: string): Promise<void> {
    return WailsServiceGateway.SyncGit(directory);
  }

  async pullGit(directory: string): Promise<void> {
    return WailsServiceGateway.PullGit(directory);
  }

  async pushGit(directory: string): Promise<void> {
    return WailsServiceGateway.PushGit(directory);
  }

  async commitGit(directory: string, message: string): Promise<void> {
    return WailsServiceGateway.CommitGit(directory, message);
  }

  async importService(directory: string): Promise<Service> {
    const result = await WailsServiceGateway.ImportService(directory);
    return result as unknown as Service;
  }

  async importCurl(serviceId: string, curlCommand: string): Promise<Service> {
    const result = await WailsServiceGateway.ImportCurl(serviceId, curlCommand);
    return result as unknown as Service;
  }

  async importSwagger(name: string, filePath: string): Promise<Service> {
    const result = await WailsServiceGateway.ImportSwagger(name, filePath);
    return result as unknown as Service;
  }
}
