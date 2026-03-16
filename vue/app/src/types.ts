export class NavItem {
    name?: string;
    id?: number;
    url?: string;

    constructor(name?: string, id?: number) {
        this.name = name;
        this.id = id;
        this.url = "/p/" + id;
    }
}