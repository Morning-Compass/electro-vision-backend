-- up.sql
CREATE TABLE roles (
    id   serial PRIMARY KEY,
    name varchar DEFAULT 'USER'::character varying NOT NULL
);

CREATE TABLE auth_users (
    id            serial PRIMARY KEY,
    username      varchar NOT NULL,
    email         varchar NOT NULL UNIQUE,
    password      varchar NOT NULL,
    created_at    timestamp NOT NULL,
    account_valid boolean NOT NULL
);

CREATE TABLE confirmation_tokens (
    id           serial PRIMARY KEY,
    user_email   varchar NOT NULL REFERENCES auth_users(email) ON UPDATE CASCADE ON DELETE CASCADE,
    token        varchar NOT NULL,
    created_at   timestamp DEFAULT now() NOT NULL,
    expires_at   timestamp NOT NULL,
    confirmed_at timestamp
);

CREATE TABLE password_reset_tokens (
    id           serial PRIMARY KEY,
    user_email   varchar NOT NULL REFERENCES auth_users(email) ON UPDATE CASCADE ON DELETE CASCADE,
    token        varchar NOT NULL,
    created_at   timestamp DEFAULT now() NOT NULL,
    expires_at   timestamp NOT NULL,
    confirmed_at timestamp
);

CREATE TABLE importance (
    id   serial PRIMARY KEY,
    name varchar(20)
);

CREATE TABLE status (
    id   serial PRIMARY KEY,
    name varchar(20) NOT NULL
);

CREATE TABLE ev_subscriptions (
    id           serial PRIMARY KEY,
    subscription varchar(20) NOT NULL
);

CREATE TABLE countries (
    id   serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    iso3 varchar(3),
    numeric_code integer
);

CREATE TABLE phone_dial_codes (
    id SERIAL PRIMARY KEY,
    code    varchar(6) NOT NULL,
    country_id serial REFERENCES countries ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE (code, country_id)
);

CREATE TABLE workspace_roles (
    id      serial PRIMARY KEY,
    user_id serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    name    varchar(50) NOT NULL
);

CREATE TABLE user_roles (
    user_id serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    role_id serial REFERENCES roles ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE workspaces (
    id                 serial PRIMARY KEY,
    plan_file_name     varchar(150) NOT NULL,
    start_date         timestamp DEFAULT now() NOT NULL,
    finish_date        timestamp,
    geolocation        varchar(40),
    owner_id           serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    ev_subscription_id serial REFERENCES ev_subscriptions ON UPDATE CASCADE ON DELETE CASCADE,
    name               varchar(60) NOT NULL
);

CREATE TABLE tasks_category (
    id           serial PRIMARY KEY,
    workspace_id serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE,
    name         varchar(50) NOT NULL
);

CREATE TABLE tasks (
    id                     serial PRIMARY KEY,
    workspace_id           serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE,
    assigner_id            serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    worker_id              serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    description            text,
    description_multimedia bytea,
    assignment_date        timestamp DEFAULT now() NOT NULL,
    due_date               timestamp,
    status_id              serial REFERENCES status ON UPDATE CASCADE ON DELETE CASCADE,
    title                  varchar(50) NOT NULL,
    category_id            serial REFERENCES tasks_category ON UPDATE CASCADE ON DELETE CASCADE,
    importance_id          serial REFERENCES importance ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE problems (
    id                 serial PRIMARY KEY,
    worker_id          serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    description        text,
    mentor_id          serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    problem_multimedia bytea
);

CREATE TABLE worker_workspace_data (
    employer_id   serial,
    user_id       serial PRIMARY KEY REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    working_since timestamp DEFAULT now() NOT NULL
);

CREATE TABLE positions (
    id           serial PRIMARY KEY,
    workspace_id serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    name         varchar(50)
);

CREATE TABLE full_users (
    user_id              serial PRIMARY KEY,
    phone                varchar(10) NOT NULL,
    phonde_dial_code_id  serial REFERENCES phone_dial_codes ON UPDATE CASCADE ON DELETE CASCADE,
    country_of_origin_id serial REFERENCES countries ON UPDATE CASCADE ON DELETE CASCADE,
    title                varchar(50),
    education            varchar(100),
    birth_date           date NOT NULL,
    account_bank_number  varchar(70),
    photo                bytea
);

CREATE TABLE workspace_users (
    id                  serial PRIMARY KEY,
    user_id             serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    workspace_id        serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE,
    plane_file_cut_name varchar(150),
    workspace_role_id   serial REFERENCES workspace_roles ON UPDATE CASCADE ON DELETE CASCADE,
    position_id         integer NULL REFERENCES positions ON UPDATE CASCADE ON DELETE CASCADE,
    checkin_time        time,
    checkout_time       time
);

CREATE TABLE conversations (
    id         serial PRIMARY KEY,
    name       varchar(70) NOT NULL,
    created_at timestamp DEFAULT now() NOT NULL
);

CREATE TABLE messages (
    id              serial PRIMARY KEY,
    conversation_id serial REFERENCES conversations ON UPDATE CASCADE ON DELETE CASCADE,
    sender_id       serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    body            text NOT NULL,
    read            boolean DEFAULT false NOT NULL,
    created_at      timestamp DEFAULT now() NOT NULL
);

CREATE TABLE conversation_participants (
    conversation_id serial REFERENCES conversations ON UPDATE CASCADE ON DELETE CASCADE,
    user_id         serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (user_id, conversation_id)
);

CREATE TABLE users_citizenships (
    user_id    serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    country_id serial REFERENCES countries ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (country_id, user_id)
);

CREATE TABLE attendance (
    id             serial PRIMARY KEY,
    user_id        serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    date           date NOT NULL,
    checkin        time NOT NULL,
    checkin_photo  bytea,
    checkout       time,
    checkout_photo bytea,
    workspace_id   serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE workspace_invitations (
    id           serial PRIMARY KEY,
    user_email   varchar NOT NULL REFERENCES auth_users(email) ON UPDATE CASCADE ON DELETE CASCADE,
    token        varchar NOT NULL,
    created_at   timestamp DEFAULT now() NOT NULL,
    expires_at   timestamp NOT NULL,
    confirmed_at timestamp,
    workspace_id serial NOT NULL REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE
);

-- Set ownership for all tables
ALTER TABLE roles OWNER TO postgres;
ALTER TABLE auth_users OWNER TO postgres;
ALTER TABLE confirmation_tokens OWNER TO postgres;
ALTER TABLE password_reset_tokens OWNER TO postgres;
ALTER TABLE importance OWNER TO postgres;
ALTER TABLE status OWNER TO postgres;
ALTER TABLE ev_subscriptions OWNER TO postgres;
ALTER TABLE countries OWNER TO postgres;
ALTER TABLE phone_dial_codes OWNER TO postgres;
ALTER TABLE workspace_roles OWNER TO postgres;
ALTER TABLE user_roles OWNER TO postgres;
ALTER TABLE workspaces OWNER TO postgres;
ALTER TABLE tasks_category OWNER TO postgres;
ALTER TABLE tasks OWNER TO postgres;
ALTER TABLE problems OWNER TO postgres;
ALTER TABLE worker_workspace_data OWNER TO postgres;
ALTER TABLE positions OWNER TO postgres;
ALTER TABLE full_users OWNER TO postgres;
ALTER TABLE workspace_users OWNER TO postgres;
ALTER TABLE conversations OWNER TO postgres;
ALTER TABLE messages OWNER TO postgres;
ALTER TABLE conversation_participants OWNER TO postgres;
ALTER TABLE users_citizenships OWNER TO postgres;
ALTER TABLE attendance OWNER TO postgres;
ALTER TABLE workspace_invitations OWNER TO postgres;
INSERT into ev_subscriptions ( subscription) VALUES ('FREE'), ('PLUS'), ('PRO'), ('ENTERPRISE');
INSERT into status (name) VALUES ('HELP_NEEDED'), ('TODO'), ('IN_PROGRESS'), ('COMPLETED'), ('CANCELED');

INSERT into roles (name) VALUES ('USER'), ('ADMIN'), ('SUPPORT');

INSERT into importance (name) VALUES ('LOW'), ('MEDIUM'), ('HIGH');
INSERT INTO countries (id, name, iso3, numeric_code) VALUES (1, 'Afghanistan', 'AFG', 4), (2, 'Albania', 'ALB', 8), (3, 'Algeria', 'DZA', 12), (4, 'American Samoa', 'ASM', 16), (5, 'Andorra', 'AND', 20), (6, 'Angola', 'AGO', 24), (7, 'Anguilla', 'AIA', 660), (8, 'Antarctica', NULL, NULL), (9, 'Antigua and Barbuda', 'ATG', 28), (10, 'Argentina', 'ARG', 32), (11, 'Armenia', 'ARM', 51), (12, 'Aruba', 'ABW', 533), (13, 'Australia', 'AUS', 36), (14, 'Austria', 'AUT', 40), (15, 'Azerbaijan', 'AZE', 31), (16, 'Bahamas', 'BHS', 44), (17, 'Bahrain', 'BHR', 48), (18, 'Bangladesh', 'BGD', 50), (19, 'Barbados', 'BRB', 52), (20, 'Belarus', 'BLR', 112), (21, 'Belgium', 'BEL', 56), (22, 'Belize', 'BLZ', 84), (23, 'Benin', 'BEN', 204), (24, 'Bermuda', 'BMU', 60), (25, 'Bhutan', 'BTN', 64), (26, 'Bolivia', 'BOL', 68), (27, 'Bosnia and Herzegovina', 'BIH', 70), (28, 'Botswana', 'BWA', 72), (29, 'Bouvet Island', NULL, NULL), (30, 'Brazil', 'BRA', 76), (31, 'British Indian Ocean Territory', NULL, NULL), (32, 'Brunei Darussalam', 'BRN', 96), (33, 'Bulgaria', 'BGR', 100), (34, 'Burkina Faso', 'BFA', 854), (35, 'Burundi', 'BDI', 108), (36, 'Cambodia', 'KHM', 116), (37, 'Cameroon', 'CMR', 120), (38, 'Canada', 'CAN', 124), (39, 'Cape Verde', 'CPV', 132), (40, 'Cayman Islands', 'CYM', 136), (41, 'Central African Republic', 'CAF', 140), (42, 'Chad', 'TCD', 148), (43, 'Chile', 'CHL', 152), (44, 'China', 'CHN', 156), (45, 'Christmas Island', NULL, NULL), (46, 'Cocos (Keeling) Islands', NULL, NULL), (47, 'Colombia', 'COL', 170), (48, 'Comoros', 'COM', 174), (49, 'Congo', 'COG', 178), (50, 'Congo, the Democratic Republic of the', 'COD', 180), (51, 'Cook Islands', 'COK', 184), (52, 'Costa Rica', 'CRI', 188), (53, 'Cote D''Ivoire', 'CIV', 384), (54, 'Croatia', 'HRV', 191), (55, 'Cuba', 'CUB', 192), (56, 'Cyprus', 'CYP', 196), (57, 'Czech Republic', 'CZE', 203), (58, 'Denmark', 'DNK', 208), (59, 'Djibouti', 'DJI', 262), (60, 'Dominica', 'DMA', 212), (61, 'Dominican Republic', 'DOM', 214), (62, 'Ecuador', 'ECU', 218), (63, 'Egypt', 'EGY', 818), (64, 'El Salvador', 'SLV', 222), (65, 'Equatorial Guinea', 'GNQ', 226), (66, 'Eritrea', 'ERI', 232), (67, 'Estonia', 'EST', 233), (68, 'Ethiopia', 'ETH', 231), (69, 'Falkland Islands (Malvinas)', 'FLK', 238), (70, 'Faroe Islands', 'FRO', 234), (71, 'Fiji', 'FJI', 242), (72, 'Finland', 'FIN', 246), (73, 'France', 'FRA', 250), (74, 'French Guiana', 'GUF', 254), (75, 'French Polynesia', 'PYF', 258), (76, 'French Southern Territories', NULL, NULL), (77, 'Gabon', 'GAB', 266), (78, 'Gambia', 'GMB', 270), (79, 'Georgia', 'GEO', 268), (80, 'Germany', 'DEU', 276), (81, 'Ghana', 'GHA', 288), (82, 'Gibraltar', 'GIB', 292), (83, 'Greece', 'GRC', 300), (84, 'Greenland', 'GRL', 304), (85, 'Grenada', 'GRD', 308), (86, 'Guadeloupe', 'GLP', 312), (87, 'Guam', 'GUM', 316), (88, 'Guatemala', 'GTM', 320), (89, 'Guinea', 'GIN', 324), (90, 'Guinea-Bissau', 'GNB', 624), (91, 'Guyana', 'GUY', 328), (92, 'Haiti', 'HTI', 332), (93, 'Heard Island and Mcdonald Islands', NULL, NULL), (94, 'Holy See (Vatican City State)', 'VAT', 336), (95, 'Honduras', 'HND', 340), (96, 'Hong Kong', 'HKG', 344), (97, 'Hungary', 'HUN', 348), (98, 'Iceland', 'ISL', 352), (99, 'India', 'IND', 356), (100, 'Indonesia', 'IDN', 360), (101, 'Iran, Islamic Republic of', 'IRN', 364), (102, 'Iraq', 'IRQ', 368), (103, 'Ireland', 'IRL', 372), (104, 'Israel', 'ISR', 376), (105, 'Italy', 'ITA', 380), (106, 'Jamaica', 'JAM', 388), (107, 'Japan', 'JPN', 392), (108, 'Jordan', 'JOR', 400), (109, 'Kazakhstan', 'KAZ', 398), (110, 'Kenya', 'KEN', 404), (111, 'Kiribati', 'KIR', 296), (112, 'Korea, Democratic People''s Republic of', 'PRK', 408), (113, 'Korea, Republic of', 'KOR', 410), (114, 'Kuwait', 'KWT', 414), (115, 'Kyrgyzstan', 'KGZ', 417), (116, 'Lao People''s Democratic Republic', 'LAO', 418), (117, 'Latvia', 'LVA', 428), (118, 'Lebanon', 'LBN', 422), (119, 'Lesotho', 'LSO', 426), (120, 'Liberia', 'LBR', 430), (121, 'Libyan Arab Jamahiriya', 'LBY', 434), (122, 'Liechtenstein', 'LIE', 438), (123, 'Lithuania', 'LTU', 440), (124, 'Luxembourg', 'LUX', 442), (125, 'Macao', 'MAC', 446), (126, 'Macedonia, the Former Yugoslav Republic of', 'MKD', 807), (127, 'Madagascar', 'MDG', 450), (128, 'Malawi', 'MWI', 454), (129, 'Malaysia', 'MYS', 458), (130, 'Maldives', 'MDV', 462), (131, 'Mali', 'MLI', 466), (132, 'Malta', 'MLT', 470), (133, 'Marshall Islands', 'MHL', 584), (134, 'Martinique', 'MTQ', 474), (135, 'Mauritania', 'MRT', 478), (136, 'Mauritius', 'MUS', 480), (137, 'Mayotte', NULL, NULL), (138, 'Mexico', 'MEX', 484), (139, 'Micronesia, Federated States of', 'FSM', 583), (140, 'Moldova, Republic of', 'MDA', 498), (141, 'Monaco', 'MCO', 492), (142, 'Mongolia', 'MNG', 496), (143, 'Montserrat', 'MSR', 500), (144, 'Morocco', 'MAR', 504), (145, 'Mozambique', 'MOZ', 508), (146, 'Myanmar', 'MMR', 104), (147, 'Namibia', 'NAM', 516), (148, 'Nauru', 'NRU', 520), (149, 'Nepal', 'NPL', 524), (150, 'Netherlands', 'NLD', 528), (151, 'Netherlands Antilles', 'ANT', 530), (152, 'New Caledonia', 'NCL', 540), (153, 'New Zealand', 'NZL', 554), (154, 'Nicaragua', 'NIC', 558), (155, 'Niger', 'NER', 562), (156, 'Nigeria', 'NGA', 566), (157, 'Niue', 'NIU', 570), (158, 'Norfolk Island', 'NFK', 574), (159, 'Northern Mariana Islands', 'MNP', 580), (160, 'Norway', 'NOR', 578), (161, 'Oman', 'OMN', 512), (162, 'Pakistan', 'PAK', 586), (163, 'Palau', 'PLW', 585), (164, 'Palestinian Territory, Occupied', NULL, NULL), (165, 'Panama', 'PAN', 591), (166, 'Papua New Guinea', 'PNG', 598), (167, 'Paraguay', 'PRY', 600), (168, 'Peru', 'PER', 604), (169, 'Philippines', 'PHL', 608), (170, 'Pitcairn', 'PCN', 612), (171, 'Poland', 'POL', 616), (172, 'Portugal', 'PRT', 620), (173, 'Puerto Rico', 'PRI', 630), (174, 'Qatar', 'QAT', 634), (175, 'Reunion', 'REU', 638), (176, 'Romania', 'ROM', 642), (177, 'Russian Federation', 'RUS', 643), (178, 'Rwanda', 'RWA', 646), (179, 'Saint Helena', 'SHN', 654), (180, 'Saint Kitts and Nevis', 'KNA', 659), (181, 'Saint Lucia', 'LCA', 662), (182, 'Saint Pierre and Miquelon', 'SPM', 666), (183, 'Saint Vincent and the Grenadines', 'VCT', 670), (184, 'Samoa', 'WSM', 882), (185, 'San Marino', 'SMR', 674), (186, 'Sao Tome and Principe', 'STP', 678), (187, 'Saudi Arabia', 'SAU', 682), (188, 'Senegal', 'SEN', 686), (189, 'Serbia and Montenegro', NULL, NULL), (190, 'Seychelles', 'SYC', 690), (191, 'Sierra Leone', 'SLE', 694), (192, 'Singapore', 'SGP', 702), (193, 'Slovakia', 'SVK', 703), (194, 'Slovenia', 'SVN', 705), (195, 'Solomon Islands', 'SLB', 90), (196, 'Somalia', 'SOM', 706), (197, 'South Africa', 'ZAF', 710), (198, 'South Georgia and the South Sandwich Islands', NULL, NULL), (199, 'Spain', 'ESP', 724), (200, 'Sri Lanka', 'LKA', 144), (201, 'Sudan', 'SDN', 736), (202, 'Suriname', 'SUR', 740), (203, 'Svalbard and Jan Mayen', 'SJM', 744), (204, 'Swaziland', 'SWZ', 748), (205, 'Sweden', 'SWE', 752), (206, 'Switzerland', 'CHE', 756), (207, 'Syrian Arab Republic', 'SYR', 760), (208, 'Taiwan, Province of China', 'TWN', 158), (209, 'Tajikistan', 'TJK', 762), (210, 'Tanzania, United Republic of', 'TZA', 834), (211, 'Thailand', 'THA', 764), (212, 'Timor-Leste', NULL, NULL), (213, 'Togo', 'TGO', 768), (214, 'Tokelau', 'TKL', 772), (215, 'Tonga', 'TON', 776), (216, 'Trinidad and Tobago', 'TTO', 780), (217, 'Tunisia', 'TUN', 788), (218, 'Turkey', 'TUR', 792), (219, 'Turkmenistan', 'TKM', 795), (220, 'Turks and Caicos Islands', 'TCA', 796), (221, 'Tuvalu', 'TUV', 798), (222, 'Uganda', 'UGA', 800), (223, 'Ukraine', 'UKR', 804), (224, 'United Arab Emirates', 'ARE', 784), (225, 'United Kingdom', 'GBR', 826), (226, 'United States', 'USA', 840), (227, 'United States Minor Outlying Islands', NULL, NULL), (228, 'Uruguay', 'URY', 858), (229, 'Uzbekistan', 'UZB', 860), (230, 'Vanuatu', 'VUT', 548), (231, 'Venezuela', 'VEN', 862), (232, 'Viet Nam', 'VNM', 704), (233, 'Virgin Islands, British', 'VGB', 92), (234, 'Virgin Islands, U.s.', 'VIR', 850), (235, 'Wallis and Futuna', 'WLF', 876), (236, 'Western Sahara', 'ESH', 732), (237, 'Yemen', 'YEM', 887), (238, 'Zambia', 'ZMB', 894), (239, 'Zimbabwe', 'ZWE', 716);
INSERT INTO phone_dial_codes (code, country_id) VALUES ('93', 1), ('355', 2), ('213', 3), ('1684', 4), ('376', 5), ('244', 6), ('1264', 7), ('0', 8), ('1268', 9), ('54', 10), ('374', 11), ('297', 12), ('61', 13), ('43', 14), ('994', 15), ('1242', 16), ('973', 17), ('880', 18), ('1246', 19), ('375', 20), ('32', 21), ('501', 22), ('229', 23), ('1441', 24), ('975', 25), ('591', 26), ('387', 27), ('267', 28), ('0', 29), ('55', 30), ('246', 31), ('673', 32), ('359', 33), ('226', 34), ('257', 35), ('855', 36), ('237', 37), ('1', 38), ('238', 39), ('1345', 40), ('236', 41), ('235', 42), ('56', 43), ('86', 44), ('61', 45), ('672', 46), ('57', 47), ('269', 48), ('242', 49), ('242', 50), ('682', 51), ('506', 52), ('225', 53), ('385', 54), ('53', 55), ('357', 56), ('420', 57), ('45', 58), ('253', 59), ('1767', 60), ('1809', 61), ('593', 62), ('20', 63), ('503', 64), ('240', 65), ('291', 66), ('372', 67), ('251', 68), ('500', 69), ('298', 70), ('679', 71), ('358', 72), ('33', 73), ('594', 74), ('689', 75), ('0', 76), ('241', 77), ('220', 78), ('995', 79), ('49', 80), ('233', 81), ('350', 82), ('30', 83), ('299', 84), ('1473', 85), ('590', 86), ('1671', 87), ('502', 88), ('224', 89), ('245', 90), ('592', 91), ('509', 92), ('0', 93), ('39', 94), ('504', 95), ('852', 96), ('36', 97), ('354', 98), ('91', 99), ('62', 100), ('98', 101), ('964', 102), ('353', 103), ('972', 104), ('39', 105), ('1876', 106), ('81', 107), ('962', 108), ('7', 109), ('254', 110), ('686', 111), ('850', 112), ('82', 113), ('965', 114), ('996', 115), ('856', 116), ('371', 117), ('961', 118), ('266', 119), ('231', 120), ('218', 121), ('423', 122), ('370', 123), ('352', 124), ('853', 125), ('389', 126), ('261', 127), ('265', 128), ('60', 129), ('960', 130), ('223', 131), ('356', 132), ('692', 133), ('596', 134), ('222', 135), ('230', 136), ('269', 137), ('52', 138), ('691', 139), ('373', 140), ('377', 141), ('976', 142), ('1664', 143), ('212', 144), ('258', 145), ('95', 146), ('264', 147), ('674', 148), ('977', 149), ('31', 150), ('599', 151), ('687', 152), ('64', 153), ('505', 154), ('227', 155), ('234', 156), ('683', 157), ('672', 158), ('1670', 159), ('47', 160), ('968', 161), ('92', 162), ('680', 163), ('970', 164), ('507', 165), ('675', 166), ('595', 167), ('51', 168), ('63', 169), ('0', 170), ('48', 171), ('351', 172), ('1787', 173), ('974', 174), ('262', 175), ('40', 176), ('70', 177), ('250', 178), ('290', 179), ('1869', 180), ('1758', 181), ('508', 182), ('1784', 183), ('684', 184), ('378', 185), ('239', 186), ('966', 187), ('221', 188), ('381', 189), ('248', 190), ('232', 191), ('65', 192), ('421', 193), ('386', 194), ('677', 195), ('252', 196), ('27', 197), ('0', 198), ('34', 199), ('94', 200), ('249', 201), ('597', 202), ('47', 203), ('268', 204), ('46', 205), ('41', 206), ('963', 207), ('886', 208), ('992', 209), ('255', 210), ('66', 211), ('670', 212), ('228', 213), ('690', 214), ('676', 215), ('1868', 216), ('216', 217), ('90', 218), ('7370', 219), ('1649', 220), ('688', 221), ('256', 222), ('380', 223), ('971', 224), ('44', 225), ('1', 226), ('1', 227), ('598', 228), ('998', 229), ('678', 230), ('58', 231), ('84', 232), ('1284', 233), ('1340', 234), ('681', 235), ('212', 236), ('967', 237), ('260', 238), ('263', 239);
